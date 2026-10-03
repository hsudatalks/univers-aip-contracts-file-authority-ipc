#![allow(dead_code)]
use async_trait::async_trait;
use bytes::Bytes;
use std::{collections::BTreeSet, sync::Arc};
use univers_aip_contracts_data::{
    file::ArtifactDescriptor,
    storage::{KvStore, KvValueCondition},
};
use univers_aip_contracts_file_authority_ipc::{
    content::retention, content_world_policy_projection as projection,
    content_world_verifier as verifier,
};
use univers_aip_lib_kv_redb::RedbKvStore;
use univers_file_governed_policy::{
    Access, ArtifactGrant, ArtifactPolicy, FilesPolicyAuthority, FilesPurpose, Material,
    PolicyError, PolicyMutation, Result, SelectedStorage, TrustedFilesAuthorizer,
    TrustedMaterialPort,
};
#[path = "retention_support.rs"]
mod canonical;
// Explicitly unsigned fixture ports, NEVER installed Auth/material authority.
struct FixtureAuth;
#[async_trait]
impl TrustedFilesAuthorizer for FixtureAuth {
    async fn authorize_access(&self, access: &Access) -> Result<()> {
        if access.principal == "principal-1" {
            Ok(())
        } else {
            Err(PolicyError::Denied(
                "unsigned fixture denies changed caller".into(),
            ))
        }
    }
    async fn authorize_policy_mutation(&self, m: &PolicyMutation) -> Result<()> {
        self.authorize_access(&m.original).await
    }
    async fn authorize_terminal_release(&self, _: &Access, proof: &[u8]) -> Result<()> {
        if proof == b"unsigned-fixture-terminal" {
            Ok(())
        } else {
            Err(PolicyError::Denied("fixture terminal denied".into()))
        }
    }
}
struct FixtureMaterial {
    kv: Arc<RedbKvStore>,
    descriptor: ArtifactDescriptor,
    bytes: Bytes,
}
#[async_trait]
impl TrustedMaterialPort for FixtureMaterial {
    async fn read_complete(
        &self,
        _: &SelectedStorage,
        _: &Access,
        _: Option<&str>,
        _: u64,
    ) -> Result<Material> {
        Ok(Material {
            descriptor: self.descriptor.clone(),
            bytes: self.bytes.clone(),
            conditions: vec![KvValueCondition {
                key: "fixture/material".into(),
                expected: self.kv.get("fixture/material").await.unwrap(),
            }],
            reservation: None,
        })
    }
    async fn require_writer_fence(&self, _: &SelectedStorage, _: &str) -> Result<()> {
        Ok(())
    }
}
pub struct Fixture {
    pub directory: tempfile::TempDir,
    pub kv: Arc<RedbKvStore>,
    pub selected: SelectedStorage,
    pub access: Access,
    pub retained: retention::Retained,
    pub terminal: retention::WorldTerminal,
    pub authority: FilesPolicyAuthority,
}
impl Fixture {
    pub async fn new(scope_padding: usize) -> Self {
        let (retained, bytes) = canonical::retained();
        let original = &retained.original;
        let c = &original.admission.correlation;
        let retention::Invocation::Spatial(invocation) = &original.invocation else {
            unreachable!()
        };
        let selected = SelectedStorage {
            organization: c.selected_world_fence.organization_id.clone(),
            world: c.selected_world_fence.world_instance_id.clone(),
            storage_identity: "fixture-original-operating-store-incarnation".into(),
        };
        // Complete Access is an owner port input. This fixture encoding is NOT a
        // guessed installed World Access codec or a reconstruction of authority.
        let mut scope = rmp_serde::to_vec_named(&c.selected_world_fence).unwrap();
        scope.extend(vec![255; scope_padding]);
        let access = Access {
            organization: selected.organization.clone(),
            world: selected.world.clone(),
            selected_world_fence: c.selected_world_fence.clone(),
            expected_material: original.material.clone(),
            runtime_identity: original.world_runtime.runtime_instance_id.clone(),
            principal: c.security_context.principal_id.clone(),
            credential: c.security_context.credential_id.clone().unwrap(),
            delegation: Some(serde_json::to_string(&c.security_context.delegation_chain).unwrap()),
            artifact_id: original.material.artifact_id.clone(),
            artifact_ref: original.material.artifact_ref.to_string(),
            purpose: FilesPurpose::SpatialBinding,
            canonical_scope: scope,
            original_context: invocation.forwarded_security_context.as_bytes().to_vec(),
            original_admission: rmp_serde::to_vec_named(&original.admission).unwrap(),
        };
        let directory = tempfile::tempdir().unwrap();
        let kv = Arc::new(RedbKvStore::open(directory.path().join("fixture.redb")).unwrap());
        let descriptor: ArtifactDescriptor = serde_json::from_value(serde_json::json!({"id":access.artifact_id,"reference":access.artifact_ref,"organization_id":selected.organization,"kind":"reality_geometry","storage_backend":"unsigned-fixture","path":"fixture/material","sha256":original.material.sha256,"size_bytes":bytes.len(),"content_type":original.material.content_type,"owner_module":null,"owner_id":null,"metadata":{"synthetic":true},"status":"active","version":1,"created_at":"2026-10-03T00:00:00Z","updated_at":"2026-10-03T00:00:00Z"})).unwrap();
        kv.put(
            "fixture/material",
            &rmp_serde::to_vec_named(&descriptor).unwrap(),
        )
        .await
        .unwrap();
        let authority = FilesPolicyAuthority::new(
            kv.clone(),
            selected.clone(),
            Arc::new(FixtureAuth),
            Arc::new(FixtureMaterial {
                kv: kv.clone(),
                descriptor,
                bytes: Bytes::from(bytes),
            }),
        )
        .unwrap();
        authority
            .mutate_policy(&PolicyMutation {
                original: access.clone(),
                expected_epoch: None,
                policy: ArtifactPolicy {
                    artifact_id: access.artifact_id.clone(),
                    artifact_ref: access.artifact_ref.clone(),
                    grants: vec![ArtifactGrant {
                        principal: access.principal.clone(),
                        credential: access.credential.clone(),
                        delegation: access.delegation.clone(),
                        purposes: BTreeSet::from([access.purpose]),
                    }],
                },
            })
            .await
            .unwrap();
        authority.acquire(&access).await.unwrap();
        let terminal = canonical::ack(&retained, false);
        Self {
            directory,
            kv,
            selected,
            access,
            retained,
            terminal,
            authority,
        }
    }
    pub fn requests(&self) -> Vec<projection::Request> {
        let retention::Request::Status { observer, .. } = canonical::status(&self.retained) else {
            unreachable!()
        };
        vec![
            verifier::Operation::VerifyAcquisition(Box::new(self.retained.original.clone())),
            verifier::Operation::VerifyOriginalWithObserver {
                retained: Box::new(self.retained.clone()),
                observer: observer.clone(),
            },
            verifier::Operation::VerifyTerminalWithObserver {
                retained: Box::new(self.retained.clone()),
                terminal: Box::new(self.terminal.clone()),
                observer,
            },
        ]
        .into_iter()
        .map(|op| projection::Request::new(verifier::Request::new(op)))
        .collect()
    }
    pub async fn projection(&self) -> projection::FullPolicyProjection {
        let (epoch, snapshot) = self
            .authority
            .held_policy_snapshot(&self.access)
            .await
            .unwrap();
        let witness_bytes = vec![
            0xc1, 0xff, 0, 0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        ];
        projection::FullPolicyProjection {
            original_operating_store_identity: self.selected.storage_identity.clone(),
            files_held_policy_epoch: epoch,
            files_held_policy_snapshot_digest:
                univers_aip_contracts_data::content_types::ContentDigest::new(canonical::hash(
                    &snapshot,
                ))
                .unwrap(),
            files_held_policy_snapshot: snapshot,
            witness: verifier::FullWitness {
                witness_digest: univers_aip_contracts_data::content_types::ContentDigest::new(
                    canonical::hash(&witness_bytes),
                )
                .unwrap(),
                witness_bytes,
                original_journal_bytes: self.terminal.original_world_journal.clone(),
                original_journal_digest: self.terminal.original_world_journal_digest.clone(),
            },
        }
    }
}
