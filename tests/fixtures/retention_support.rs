#![allow(dead_code, unused_imports)]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use univers_aip_contracts_data::content_types::{ContentDescriptor, ContentDigest, ContentScope};
use univers_aip_contracts_file_authority_ipc::content::retention::{CAPABILITY, SCHEMA};
use univers_aip_contracts_file_authority_ipc::content::{self, retention::*};
use univers_aip_contracts_world_ipc::spatial_mutation::*;

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("retention_original.json")).unwrap()
}
pub fn hash(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
pub fn rehash(original: &mut Original) {
    let Invocation::Spatial(value) = &mut original.invocation else {
        unreachable!()
    };
    let SpatialBindingOperation::Apply(request) = &mut value.operation else {
        unreachable!()
    };
    request.request_digest = request.canonical_request_digest().unwrap();
    original.admission.correlation.request_digest = request.request_digest.clone();
    let c = &original.admission.correlation;
    let mut input = json!({"provider":c.provider_id,"action":c.action_id,"identity":c.security_context,
        "world":c.selected_world_fence,"payload":value.operation.admission_payload().unwrap()});
    input.sort_all_objects();
    original.admission.correlation.invocation_digest =
        format!("{:x}", Sha256::digest(serde_json::to_vec(&input).unwrap()));
}
pub fn original() -> (Original, Vec<u8>) {
    let f = fixture();
    let bytes: Vec<u8> = (0..128).map(|i| (i % 256) as u8).collect();
    let mut operation: SpatialBindingOperation =
        serde_json::from_value(f["operationPayload"].clone()).unwrap();
    let SpatialBindingOperation::Apply(request) = &mut operation else {
        unreachable!()
    };
    request.expected_asset.sha256 = hash(&bytes);
    let material = request.expected_asset.clone();
    let mut original = Original {
        invocation: Invocation::Spatial(Box::new(SpatialBindingInvocation {
            schema_version: univers_aip_contracts_world_ipc::spatial_mutation::SCHEMA.into(),
            operation,
            forwarded_security_context: f["notSigningProofs"]["context"].as_str().unwrap().into(),
            runtime_action_admission: f["notSigningProofs"]["action"].as_str().unwrap().into(),
        })),
        admission: serde_json::from_value(f["admission"].clone()).unwrap(),
        world_runtime: serde_json::from_value(f["assignedKernel"].clone()).unwrap(),
        material,
        content: ContentDescriptor {
            scope: ContentScope::new("org-1", "world-1").unwrap(),
            digest: ContentDigest::new(hash(&bytes)).unwrap(),
            size_bytes: bytes.len() as u64,
        },
    };
    rehash(&mut original);
    original.validate().unwrap();
    (original, bytes)
}
pub fn retained() -> (Retained, Vec<u8>) {
    let (original, bytes) = original();
    (
        Retained {
            original,
            physical: PhysicalStore {
                storage_id: "content-root-1".into(),
                incarnation_id: "original-store-1".into(),
                journal_id: "original-journal-1".into(),
                original_identity_digest: ContentDigest::new(hash(b"full physical identity"))
                    .unwrap(),
            },
            generation: 1,
            original_record_digest: ContentDigest::new(hash(b"full original pin and policy"))
                .unwrap(),
        },
        bytes,
    )
}
pub fn ack(retained: &Retained, accept: bool) -> WorldTerminal {
    let mut value: SpatialBindingReconciliationAck =
        serde_json::from_value(fixture()["commitAck"].clone()).unwrap();
    value.admission = retained.original.admission.clone();
    value.kernel = retained.original.world_runtime.clone();
    let SpatialBindingQuiescentOutcome::Terminal { receipt, .. } = &mut value.outcome else {
        unreachable!()
    };
    receipt.request_digest = value.admission.correlation.request_digest.clone();
    receipt.expected_asset = retained.original.material.clone();
    if !accept {
        receipt.outcome = SpatialBindingOutcome::Rejected {
            code: SpatialBindingErrorCode::WorldSpatialInvalidRequest,
        };
        value.acknowledgement = SpatialBindingAcknowledgementKind::AbortAcknowledged;
    }
    let journal = b"synthetic complete original World terminal journal; not proof".to_vec();
    WorldTerminal {
        acknowledgement: SpatialReconciliationAckFrame {
            schema_version: univers_aip_contracts_world_ipc::spatial_reconciliation::SCHEMA.into(),
            acknowledgement: value,
        },
        original_world_journal_digest: ContentDigest::new(hash(&journal)).unwrap(),
        original_world_journal: journal,
    }
}
pub fn reference(retained: &Retained) -> AcceptedReference {
    AcceptedReference {
        reference_id: "bundle-1".into(),
        generation: 1,
        accepted: ack(retained, true),
    }
}
pub fn removal(retained: &Retained) -> ReferenceRemoval {
    let journal = b"synthetic FULL original World reference removal journal; not proof".to_vec();
    ReferenceRemoval {
        reference: reference(retained),
        removal_id: "original-removal-1".into(),
        world_runtime: retained.original.world_runtime.clone(),
        original_world_journal_digest: ContentDigest::new(hash(&journal)).unwrap(),
        original_world_journal: journal,
    }
}
pub fn drain(retained: &Retained) -> Request {
    Request::Drain {
        retained: Box::new(retained.clone()),
        request: SpatialReconciliationRequestFrame {
            schema_version: univers_aip_contracts_world_ipc::spatial_reconciliation::SCHEMA.into(),
            request: SpatialBindingReconciliationRequest {
                admission: retained.original.admission.clone(),
                kernel: retained.original.world_runtime.clone(),
                intent: SpatialBindingReconciliationIntent::Drain,
            },
        },
    }
}
pub fn status(retained: &Retained) -> Request {
    let mut request = json!({"schemaVersion":"ark.world-spatial-binding/v1", "operationId":"placement-1", "idempotencyKey":"placement-1-key",
        "requestDigest":retained.original.admission.correlation.request_digest,"organizationId":"org-1","worldInstanceId":"world-1", "expectedWorldRevision":12, "readOnly":true});
    let operation =
        SpatialBindingOperation::GetOrResume(serde_json::from_value(request.take()).unwrap());
    Request::Status {
        retained: Box::new(retained.clone()),
        observer: Invocation::Spatial(Box::new(SpatialBindingInvocation {
            schema_version: univers_aip_contracts_world_ipc::spatial_mutation::SCHEMA.into(),
            operation,
            forwarded_security_context: "separate unsigned observer context".into(),
            runtime_action_admission: "separate unsigned readonly action".into(),
        })),
    }
}
