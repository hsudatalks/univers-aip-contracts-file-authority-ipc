#![cfg(feature = "retention")]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use univers_aip_contracts_data::content_types::{ContentDescriptor, ContentDigest, ContentScope};
use univers_aip_contracts_file_authority_ipc::content::retention::{CAPABILITY, SCHEMA};
use univers_aip_contracts_file_authority_ipc::content::{self, retention::*};
use univers_aip_contracts_world_ipc::spatial_mutation::*;

fn fixture() -> Value {
    serde_json::from_str(include_str!("fixtures/retention_original.json")).unwrap()
}
fn hash(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn rehash(original: &mut Original) {
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
fn original() -> (Original, Vec<u8>) {
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
fn retained() -> (Retained, Vec<u8>) {
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
fn ack(retained: &Retained, accept: bool) -> WorldTerminal {
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
fn reference(retained: &Retained) -> AcceptedReference {
    AcceptedReference {
        reference_id: "bundle-1".into(),
        generation: 1,
        accepted: ack(retained, true),
    }
}
fn removal(retained: &Retained) -> ReferenceRemoval {
    let journal = b"synthetic FULL original World reference removal journal; not proof".to_vec();
    ReferenceRemoval {
        reference: reference(retained),
        removal_id: "original-removal-1".into(),
        world_runtime: retained.original.world_runtime.clone(),
        original_world_journal_digest: ContentDigest::new(hash(&journal)).unwrap(),
        original_world_journal: journal,
    }
}
fn drain(retained: &Retained) -> Request {
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
fn status(retained: &Retained) -> Request {
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
#[test]
fn complete_original_body_and_all_lifecycle_wire_roundtrip_without_projection() {
    let (retained, bytes) = retained();
    let requests = vec![
        Request::Probe,
        Request::Acquire(Box::new(retained.original.clone())),
        Request::Restore(Box::new(retained.clone())),
        status(&retained),
        drain(&retained),
        Request::Terminal {
            retained: Box::new(retained.clone()),
            acknowledgement: ack(&retained, false),
        },
        Request::RetainReference {
            retained: Box::new(retained.clone()),
            reference: reference(&retained),
        },
        Request::ReleaseReference {
            retained: Box::new(retained.clone()),
            removal: removal(&retained),
        },
    ];
    let replies = vec![
        Reply::Protocol(Protocol {
            schema_version: SCHEMA.into(),
            capabilities: vec![CAPABILITY.into()],
        }),
        Reply::Body {
            retained: Box::new(retained.clone()),
            bytes: bytes.clone(),
        },
        Reply::Body {
            retained: Box::new(retained.clone()),
            bytes,
        },
        Reply::Status {
            retained: Box::new(retained.clone()),
            state: Box::new(State::Held),
        },
        Reply::DrainPending(Box::new(retained.clone())),
        Reply::Terminal {
            retained: Box::new(retained.clone()),
            acknowledgement: ack(&retained, false),
        },
        Reply::Referenced {
            retained: Box::new(retained.clone()),
            reference: reference(&retained),
        },
        Reply::ReferenceReleased {
            retained: Box::new(retained.clone()),
            removal: removal(&retained),
        },
    ];
    for (request, reply) in requests.iter().zip(&replies) {
        request.validate().unwrap();
        reply.validate_for(request).unwrap();
        let named = rmp_serde::to_vec_named(request).unwrap();
        assert_eq!(Request::decode_named(&named).unwrap(), *request);
        assert_eq!(
            Request::decode_json(&serde_json::to_vec(request).unwrap()).unwrap(),
            *request
        );
        assert_eq!(
            Reply::decode_named(&rmp_serde::to_vec_named(reply).unwrap()).unwrap(),
            *reply
        );
        assert_eq!(
            Reply::decode_json(&serde_json::to_vec(reply).unwrap()).unwrap(),
            *reply
        );
        let outer = content::Request::Retention(Box::new(request.clone()));
        let typed: content::Request =
            rmp_serde::from_slice(&rmp_serde::to_vec_named(&outer).unwrap()).unwrap();
        assert_eq!(
            serde_json::to_value(outer).unwrap(),
            serde_json::to_value(typed).unwrap()
        );
    }
}
#[test]
fn acquired_inputs_reject_cross_scope_changed_proof_empty_invalid_reference_digest_size() {
    let (original, _) = original();
    for (pointer, value) in [
        ("/content/scope/organizationId", json!("other-org")),
        ("/content/scope/worldInstanceId", json!("other-world")),
        ("/content/sizeBytes", json!(0)),
        ("/content/sizeBytes", json!(65 * 1024 * 1024)),
        ("/material/artifactRef", json!("https://not-an-artifact")),
        ("/material/artifactId", json!("other-object")),
        ("/content/digest", json!("bad-digest")),
        ("/material/sha256", json!(hash(b"changed-body"))),
        (
            "/admission/correlation/securityContext/credentialId",
            Value::Null,
        ),
        (
            "/admission/correlation/securityContext/principalId",
            json!("other-caller"),
        ),
        (
            "/admission/correlation/securityContext/delegationChain/0/principalId",
            json!("other-delegate"),
        ),
        (
            "/admission/correlation/selectedWorldFence/revision",
            json!(13),
        ),
        ("/worldRuntime/providerId", json!("other-provider")),
        ("/worldRuntime/runtimeInstanceId", json!("")),
        (
            "/invocation/value/forwardedSecurityContext",
            json!("changed-context"),
        ),
        (
            "/invocation/value/runtimeActionAdmission",
            json!("changed-proof"),
        ),
    ] {
        let mut candidate = serde_json::to_value(&original).unwrap();
        *candidate.pointer_mut(pointer).unwrap() = value;
        assert!(
            serde_json::from_value::<Original>(candidate)
                .map(|v| v.validate().is_err())
                .unwrap_or(true),
            "{pointer}"
        );
    }
}
#[test]
fn expired_original_cannot_acquire_but_original_drain_does_not_renew_expiry() {
    let (retained, _) = retained();
    let deadline = retained.original.admission.not_after_unix_ms;
    retained
        .original
        .validate_acquisition_at(deadline - 1)
        .unwrap();
    assert!(retained.original.validate_acquisition_at(deadline).is_err());
    drain(&retained).validate().unwrap();
    Request::Terminal {
        retained: Box::new(retained.clone()),
        acknowledgement: ack(&retained, false),
    }
    .validate()
    .unwrap();
    let mut changed = retained.clone();
    changed.original.admission.not_after_unix_ms += 1;
    assert!(changed.validate_for(&retained).is_err());
    let mut changed = retained.clone();
    changed.generation += 1;
    assert!(changed.validate_for(&retained).is_err());
    let mut changed = retained.clone();
    changed.physical.incarnation_id = "copied-store".into();
    assert!(changed.validate_for(&retained).is_err());
}
#[test]
fn status_requires_pure_read_and_acquisition_does_not_accept_it() {
    let (retained, _) = retained();
    status(&retained).validate().unwrap();
    let Request::Status { observer, .. } = status(&retained) else {
        unreachable!()
    };
    let mut original = retained.original.clone();
    original.invocation = observer;
    assert!(original.validate().is_err());
    let mut bad = status(&retained);
    let Request::Status {
        observer: Invocation::Spatial(value),
        ..
    } = &mut bad
    else {
        unreachable!()
    };
    let SpatialBindingOperation::GetOrResume(r) = &mut value.operation else {
        unreachable!()
    };
    r.read_only = Some(false);
    assert!(bad.validate().is_err());
    let mut bad = status(&retained);
    let Request::Status {
        observer: Invocation::Spatial(value),
        ..
    } = &mut bad
    else {
        unreachable!()
    };
    let SpatialBindingOperation::GetOrResume(r) = &mut value.operation else {
        unreachable!()
    };
    r.world_instance_id = "other-world".into();
    assert!(bad.validate().is_err());
}
#[test]
fn partial_extra_tail_tampered_body_and_reply_generation_mismatch_are_denied() {
    let (retained, bytes) = retained();
    let request = Request::Restore(Box::new(retained.clone()));
    for changed in [
        bytes[..bytes.len() - 1].to_vec(),
        [bytes.clone(), vec![0]].concat(),
        {
            let mut b = bytes.clone();
            *b.last_mut().unwrap() ^= 1;
            b
        },
    ] {
        assert!(Reply::Body {
            retained: Box::new(retained.clone()),
            bytes: changed
        }
        .validate_for(&request)
        .is_err());
    }
    let mut successor = retained.clone();
    successor.generation += 1;
    assert!(Reply::Body {
        retained: Box::new(successor),
        bytes
    }
    .validate_for(&request)
    .is_err());
}
#[test]
fn accepted_ack_never_releases_and_old_false_terminal_ack_never_matches_successor() {
    let (retained, _) = retained();
    assert!(Request::Terminal {
        retained: Box::new(retained.clone()),
        acknowledgement: ack(&retained, true)
    }
    .validate()
    .is_err());
    let good = ack(&retained, false);
    for pointer in [
        "/acknowledgement/acknowledgement/admission/correlation/actionProofDigest",
        "/acknowledgement/acknowledgement/admission/correlation/forwardedContextProofDigest",
        "/acknowledgement/acknowledgement/admission/correlation/requestDigest",
    ] {
        let mut value = serde_json::to_value(&good).unwrap();
        *value.pointer_mut(pointer).unwrap() = json!(hash(b"wrong"));
        let bad = serde_json::from_value(value).unwrap();
        assert!(Request::Terminal {
            retained: Box::new(retained.clone()),
            acknowledgement: bad
        }
        .validate()
        .is_err());
    }
    let mut stale = good.clone();
    stale
        .acknowledgement
        .acknowledgement
        .kernel
        .runtime_instance_id = "old-incarnation".into();
    assert!(Request::Terminal {
        retained: Box::new(retained.clone()),
        acknowledgement: stale
    }
    .validate()
    .is_err());
    let mut staged = good;
    staged.acknowledgement.acknowledgement.outcome = SpatialBindingQuiescentOutcome::NotReserved {};
    assert!(Request::Terminal {
        retained: Box::new(retained.clone()),
        acknowledgement: staged
    }
    .validate()
    .is_err());
    let mut changed = ack(&retained, false);
    let SpatialBindingQuiescentOutcome::Terminal { receipt, .. } =
        &mut changed.acknowledgement.acknowledgement.outcome
    else {
        unreachable!()
    };
    receipt.source.source_revision = "changed-full-result".into();
    assert!(Request::Terminal {
        retained: Box::new(retained),
        acknowledgement: changed
    }
    .validate()
    .is_err());
}
#[test]
fn reference_removal_requires_exact_generation_full_accepted_result_and_original_journal() {
    let (retained, _) = retained();
    let reference = reference(&retained);
    let removal = removal(&retained);
    removal.validate_for(&retained, &reference).unwrap();
    let mut successor = reference.clone();
    successor.generation += 1;
    assert!(removal.validate_for(&retained, &successor).is_err());
    let mut bad = removal.clone();
    bad.original_world_journal.clear();
    assert!(bad.validate_for(&retained, &reference).is_err());
    let mut bad = removal.clone();
    *bad.original_world_journal.last_mut().unwrap() ^= 1;
    assert!(bad.validate_for(&retained, &reference).is_err());
    let mut bad = reference.clone();
    bad.reference_id = "unaccepted-binding".into();
    assert!(bad.validate_for(&retained).is_err());
    let mut bad = reference.clone();
    bad.accepted = ack(&retained, false);
    assert!(bad.validate_for(&retained).is_err());
    let mut bad = reference;
    bad.generation = 0;
    assert!(bad.validate_for(&retained).is_err());
}
#[test]
fn strict_decoders_reject_duplicate_unknown_positional_trailing_malformed_oversized() {
    let (retained, _) = retained();
    let request = Request::Restore(Box::new(retained));
    let mut trailing = rmp_serde::to_vec_named(&request).unwrap();
    trailing.push(0);
    assert!(Request::decode_named(&trailing).is_err());
    assert!(Request::decode_named(&rmp_serde::to_vec(&request).unwrap()).is_err());
    assert!(Request::decode_json(b"{\"operation\":\"probe\",\"operation\":\"probe\"}").is_err());
    assert!(Request::decode_json(b"{\"operation\":\"probe\"} {}").is_err());
    assert!(Request::decode_named(&[0xc1]).is_err());
    assert!(Request::decode_named(&vec![0; MAX_REQUEST_BYTES + 1]).is_err());
    let mut value = serde_json::to_value(&request).unwrap();
    value["value"]["original"]["content"]["scope"]["ignoredGrant"] = json!(true);
    assert!(Request::decode_named(&rmp_serde::to_vec_named(&value).unwrap()).is_err());
    assert!(Request::decode_json(&serde_json::to_vec(&value).unwrap()).is_err());
    let json = serde_json::to_string(&request)
        .unwrap()
        .replace("\"generation\":1", "\"generation\":1,\"generation\":1");
    assert!(Request::decode_json(json.as_bytes()).is_err());
    assert!(Protocol {
        schema_version: SCHEMA.into(),
        capabilities: vec![]
    }
    .require_supported()
    .is_err());
}

#[test]
fn governed_source_full_original_result_reference_and_readonly_status_roundtrip() {
    let source: Value =
        serde_json::from_str(include_str!("fixtures/retention_source_original.json")).unwrap();
    let (mut retained, _) = retained();
    retained.original.admission =
        serde_json::from_value(source["sourceCommitAck"]["admission"].clone()).unwrap();
    let operation: SubjectSourceOperation =
        serde_json::from_value(source["selectPayload"].clone()).unwrap();
    let SubjectSourceOperation::SubjectSourceSelect(request) = &operation else {
        unreachable!()
    };
    retained.original.material.sha256 = request.source.content_digest.clone();
    retained.original.material.size_bytes = request.source.size_bytes;
    retained.original.content.digest =
        ContentDigest::new(request.source.content_digest.clone()).unwrap();
    retained.original.content.size_bytes = request.source.size_bytes;
    let context = "synthetic exact original source context; not proof";
    let action = "synthetic exact original source action; not proof";
    retained
        .original
        .admission
        .correlation
        .forwarded_context_proof_digest = hash(context.as_bytes());
    retained.original.admission.correlation.action_proof_digest = hash(action.as_bytes());
    retained.original.invocation = Invocation::SubjectSource(Box::new(SubjectSourceInvocation {
        schema_version: univers_aip_contracts_world_ipc::subject_source::SCHEMA.into(),
        operation,
        forwarded_security_context: context.into(),
        runtime_action_admission: action.into(),
    }));
    retained.validate().unwrap();
    let acquire = Request::Acquire(Box::new(retained.original.clone()));
    assert_eq!(
        Request::decode_named(&rmp_serde::to_vec_named(&acquire).unwrap()).unwrap(),
        acquire
    );
    let mut acknowledgement: SpatialBindingReconciliationAck =
        serde_json::from_value(source["sourceCommitAck"].clone()).unwrap();
    acknowledgement.admission = retained.original.admission.clone();
    let frame = SpatialReconciliationAckFrame {
        schema_version: univers_aip_contracts_world_ipc::spatial_reconciliation::SCHEMA.into(),
        acknowledgement,
    };
    let reference = AcceptedReference {
        reference_id: source["selection"]["selectionId"].as_str().unwrap().into(),
        generation: 1,
        accepted: WorldTerminal {
            acknowledgement: frame,
            original_world_journal: b"synthetic complete original source journal".to_vec(),
            original_world_journal_digest: ContentDigest::new(hash(
                b"synthetic complete original source journal",
            ))
            .unwrap(),
        },
    };
    reference.validate_for(&retained).unwrap();
    let request = Request::RetainReference {
        retained: Box::new(retained.clone()),
        reference: reference.clone(),
    };
    let reply = Reply::Referenced {
        retained: Box::new(retained.clone()),
        reference,
    };
    reply.validate_for(&request).unwrap();
    assert_eq!(
        Reply::decode_named(&rmp_serde::to_vec_named(&reply).unwrap()).unwrap(),
        reply
    );
    let observer = Invocation::SubjectSource(Box::new(SubjectSourceInvocation {
        schema_version: univers_aip_contracts_world_ipc::subject_source::SCHEMA.into(),
        operation: SubjectSourceOperation::SubjectSourceRead(
            serde_json::from_value(source["readRequest"].clone()).unwrap(),
        ),
        forwarded_security_context: "unsigned separate readonly source context".into(),
        runtime_action_admission: "unsigned separate readonly source action".into(),
    }));
    let request = Request::Status {
        retained: Box::new(retained),
        observer,
    };
    request.validate().unwrap();
    assert_eq!(
        Request::decode_named(&rmp_serde::to_vec_named(&request).unwrap()).unwrap(),
        request
    );
}

#[test]
fn strict_named_decoder_rejects_duplicate_nested_fields_before_value_projection() {
    use serde::{ser::SerializeMap, Serialize};
    struct Duplicate<'a>(&'a Retained);
    impl Serialize for Duplicate<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut map = serializer.serialize_map(Some(5))?;
            map.serialize_entry("original", &self.0.original)?;
            map.serialize_entry("physical", &self.0.physical)?;
            map.serialize_entry("generation", &self.0.generation)?;
            map.serialize_entry("originalRecordDigest", &self.0.original_record_digest)?;
            map.serialize_entry("generation", &self.0.generation)?;
            map.end()
        }
    }
    #[derive(Serialize)]
    struct Restore<'a> {
        operation: &'static str,
        value: Duplicate<'a>,
    }
    let (retained, _) = retained();
    let bytes = rmp_serde::to_vec_named(&Restore {
        operation: "restore",
        value: Duplicate(&retained),
    })
    .unwrap();
    assert!(Request::decode_named(&bytes).is_err());
}

#[test]
fn complete_content_envelope_strict_decode_and_actual_recorded_generation_gate() {
    let (retained, bytes) = retained();
    let request = Request::Restore(Box::new(retained.clone()));
    let outer = content::Request::Retention(Box::new(request.clone()));
    assert_eq!(
        Request::decode_content_named(&rmp_serde::to_vec_named(&outer).unwrap()).unwrap(),
        request
    );
    assert_eq!(
        Request::decode_content_json(&serde_json::to_vec(&outer).unwrap()).unwrap(),
        request
    );
    let reply = Reply::Body {
        retained: Box::new(retained.clone()),
        bytes,
    };
    let outer = content::Reply::Retention(Box::new(reply.clone()));
    assert_eq!(
        Reply::decode_content_named(&rmp_serde::to_vec_named(&outer).unwrap()).unwrap(),
        reply
    );
    assert_eq!(
        Reply::decode_content_json(&serde_json::to_vec(&outer).unwrap()).unwrap(),
        reply
    );
    let mut unknown = serde_json::to_value(&outer).unwrap();
    unknown["value"]["value"]["retained"]["original"]["content"]["ignoredAuthority"] = json!(true);
    assert!(Reply::decode_content_named(&rmp_serde::to_vec_named(&unknown).unwrap()).is_err());
    let terminal = Request::Terminal {
        retained: Box::new(retained.clone()),
        acknowledgement: ack(&retained, false),
    };
    terminal.validate_recorded(&retained, None).unwrap();
    let mut successor = retained.clone();
    successor.generation += 1;
    assert!(terminal.validate_recorded(&successor, None).is_err());
    assert!(terminal
        .validate_recorded(&retained, Some(&reference(&retained)))
        .is_err());
    let release = Request::ReleaseReference {
        retained: Box::new(retained.clone()),
        removal: removal(&retained),
    };
    assert!(release.validate_recorded(&retained, None).is_err());
    let reference = reference(&retained);
    release
        .validate_recorded(&retained, Some(&reference))
        .unwrap();
    let mut successor = reference;
    successor.generation += 1;
    assert!(release
        .validate_recorded(&retained, Some(&successor))
        .is_err());
}

#[test]
fn terminal_journal_is_complete_and_digest_bound_to_original_ack() {
    let (retained, _) = retained();
    let valid = ack(&retained, false);
    for bytes in [
        Vec::new(),
        b"different original journal".to_vec(),
        vec![0; MAX_JOURNAL_BYTES + 1],
    ] {
        let mut changed = valid.clone();
        changed.original_world_journal = bytes;
        assert!(changed.validate_for(&retained).is_err());
    }
    let mut changed = valid;
    changed.original_world_journal_digest =
        ContentDigest::new(hash(b"different journal digest")).unwrap();
    assert!(changed.validate_for(&retained).is_err());
}
