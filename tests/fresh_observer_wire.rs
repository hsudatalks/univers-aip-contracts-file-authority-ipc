#![cfg(feature = "world-verifier")]
use serde::{ser::SerializeMap, Serialize};
use serde_json::{json, Value};
use univers_aip_contracts_data::content_types::ContentDigest;
use univers_aip_contracts_file_authority_ipc::{content::retention, content_world_verifier::*};
#[path = "fixtures/retention_support.rs"]
mod support;
fn cases() -> Vec<Request> {
    let (mut retained, _) = support::retained();
    retained.generation = u64::MAX;
    let retention::Request::Status { observer, .. } = support::status(&retained) else {
        unreachable!()
    };
    vec![
        Request::new(Operation::VerifyOriginalWithObserver {
            retained: Box::new(retained.clone()),
            observer: observer.clone(),
        }),
        Request::new(Operation::VerifyTerminalWithObserver {
            retained: Box::new(retained.clone()),
            terminal: Box::new(support::ack(&retained, false)),
            observer,
        }),
    ]
}
fn witness() -> FullWitness {
    let bytes = vec![
        0xc1, 0xff, 0, 0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    ];
    let journal = vec![0xff, 0, 0xc1, 0x80, 0xfe];
    FullWitness {
        witness_digest: ContentDigest::new(support::hash(&bytes)).unwrap(),
        witness_bytes: bytes,
        original_journal_digest: ContentDigest::new(support::hash(&journal)).unwrap(),
        original_journal_bytes: journal,
    }
}
#[test]
fn expired_original_separate_readonly_observer_full_result_is_always_untrusted() {
    for request in cases() {
        request.validate().unwrap();
        let (retained, observer) = match &request.operation {
            Operation::VerifyOriginalWithObserver { retained, observer }
            | Operation::VerifyTerminalWithObserver {
                retained, observer, ..
            } => (retained, observer),
            _ => unreachable!(),
        };
        let before = rmp_serde::to_vec_named(&retained.original).unwrap();
        assert!(retained
            .original
            .validate_acquisition_at(retained.original.admission.not_after_unix_ms)
            .is_err());
        assert!(observer.is_pure_read());
        assert!(request.validate_acquisition_at(0).is_err());
        let terminal = match &request.operation {
            Operation::VerifyTerminalWithObserver { terminal, .. } => Some(terminal.as_ref()),
            _ => None,
        };
        request.validate_recorded(retained, terminal).unwrap();
        for named in [true, false] {
            let bytes = if named {
                rmp_serde::to_vec_named(&request).unwrap()
            } else {
                serde_json::to_vec(&request).unwrap()
            };
            let decoded = if named {
                Request::decode_named(&bytes)
            } else {
                Request::decode_json(&bytes)
            }
            .unwrap();
            assert_eq!(decoded, request);
            for status in [
                Status::UntrustedWitness(Box::new(witness())),
                Status::Denied {
                    reason: "fresh caller credential expired/revoked; not authenticated".into(),
                },
                Status::Unavailable {
                    reason: "original store unavailable".into(),
                },
                Status::RecoveryRequired {
                    reason: "owned and queued terminal uncertain".into(),
                },
            ] {
                let reply = Reply {
                    request: request.clone(),
                    status,
                };
                let bytes = if named {
                    rmp_serde::to_vec_named(&reply).unwrap()
                } else {
                    serde_json::to_vec(&reply).unwrap()
                };
                let decoded = if named {
                    Reply::decode_named(&bytes)
                } else {
                    Reply::decode_json(&bytes)
                }
                .unwrap();
                decoded.validate_for(&request).unwrap();
                assert_eq!(decoded, reply);
            }
        }
        assert_eq!(rmp_serde::to_vec_named(&retained.original).unwrap(), before);
    }
}
#[test]
fn observer_is_mandatory_pure_read_same_family_and_exact_original_input_scope_fence() {
    for request in cases() {
        for (path, value) in [
            (
                "/operation/value/observer/value/operation/request/operationId",
                json!("other-original"),
            ),
            (
                "/operation/value/observer/value/operation/request/idempotencyKey",
                json!("other-idempotency"),
            ),
            (
                "/operation/value/observer/value/operation/request/requestDigest",
                json!(support::hash(b"other original input")),
            ),
            (
                "/operation/value/observer/value/operation/request/organizationId",
                json!("other-org"),
            ),
            (
                "/operation/value/observer/value/operation/request/worldInstanceId",
                json!("other-world"),
            ),
            (
                "/operation/value/observer/value/operation/request/expectedWorldRevision",
                json!(13),
            ),
            (
                "/operation/value/observer/value/operation/request/readOnly",
                json!(false),
            ),
            (
                "/operation/value/observer/value/forwardedSecurityContext",
                json!(""),
            ),
            (
                "/operation/value/observer/value/runtimeActionAdmission",
                json!(""),
            ),
        ] {
            let mut candidate = serde_json::to_value(&request).unwrap();
            *candidate.pointer_mut(path).unwrap() = value;
            assert!(Request::decode_json(&serde_json::to_vec(&candidate).unwrap()).is_err());
            assert!(Request::decode_named(&rmp_serde::to_vec_named(&candidate).unwrap()).is_err());
        }
        let mut missing = serde_json::to_value(&request).unwrap();
        missing["operation"]["value"]
            .as_object_mut()
            .unwrap()
            .remove("observer");
        assert!(Request::decode_json(&serde_json::to_vec(&missing).unwrap()).is_err());
        assert!(Request::decode_named(&rmp_serde::to_vec_named(&missing).unwrap()).is_err());
        let mut mutation = serde_json::to_value(&request).unwrap();
        let retained = &mutation["operation"]["value"]["retained"];
        mutation["operation"]["value"]["observer"] = retained["original"]["invocation"].clone();
        assert!(Request::decode_json(&serde_json::to_vec(&mutation).unwrap()).is_err());
    }
}
#[test]
fn full_current_headers_old_caller_credential_delegation_and_full_result_are_independent() {
    for request in cases() {
        for path in [
            "/operation/value/observer/value/forwardedSecurityContext",
            "/operation/value/observer/value/runtimeActionAdmission",
            "/operation/value/retained/original/invocation/value/forwardedSecurityContext",
            "/operation/value/retained/original/invocation/value/runtimeActionAdmission",
            "/operation/value/retained/original/admission/correlation/securityContext/principalId",
            "/operation/value/retained/original/admission/correlation/securityContext/credentialId",
            "/operation/value/retained/original/admission/correlation/securityContext/delegationChain/0/principalId",
            "/operation/value/retained/original/admission/correlation/actionId",
            "/operation/value/retained/original/admission/correlation/providerId",
        ] {
            let mut candidate = serde_json::to_value(&request).unwrap();
            *candidate.pointer_mut(path).unwrap() = json!("changed caller/credential/delegation/action/input");
            if let Ok(changed) = serde_json::from_value::<Request>(candidate) {
                let reply = Reply { request: changed, status: Status::Denied { reason: "correlation or current scoped Auth denied".into() } };
                assert!(reply.validate_for(&request).is_err());
            }
        }
        // Opaque proof bytes are NOT parsed/authenticated here. Current expiry and
        // revoked permissions are real World decisions; denial never becomes allow.
        let reply = Reply {
            request: request.clone(),
            status: Status::Denied {
                reason: "fresh observer expired or current permission revoked".into(),
            },
        };
        let received = Reply::decode_named(&rmp_serde::to_vec_named(&reply).unwrap()).unwrap();
        received.validate_for(&request).unwrap();
        assert!(matches!(received.status, Status::Denied { .. }));
    }
}
struct DuplicateObserver<'a>(&'a Request);
impl Serialize for DuplicateObserver<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value = serde_json::to_value(self.0).unwrap();
        let mut map = serializer.serialize_map(Some(6))?;
        for (key, value) in value.as_object().unwrap() {
            map.serialize_entry(key, value)?;
        }
        map.serialize_entry("operation", &value["operation"])?;
        map.end()
    }
}
#[test]
fn new_operations_strict_unknown_duplicate_trailing_bounds_and_witness_tamper() {
    for request in cases() {
        assert!(Request::decode_named(
            &rmp_serde::to_vec_named(&DuplicateObserver(&request)).unwrap()
        )
        .is_err());
        let mut value = serde_json::to_value(&request).unwrap();
        value["operation"]["value"]["observer"]["value"]["serializedAllow"] = json!(true);
        assert!(Request::decode_named(&rmp_serde::to_vec_named(&value).unwrap()).is_err());
        assert!(Request::decode_json(&serde_json::to_vec(&value).unwrap()).is_err());
        let duplicate = serde_json::to_string(&request)
            .unwrap()
            .replace("\"observer\":", "\"observer\":null,\"observer\":");
        assert!(Request::decode_json(duplicate.as_bytes()).is_err());
        let mut trailing = rmp_serde::to_vec_named(&request).unwrap();
        trailing.push(0);
        assert!(Request::decode_named(&trailing).is_err());
        assert!(Request::decode_named(&rmp_serde::to_vec(&request).unwrap()).is_err());
    }
    assert!(Request::decode_named(&vec![0; MAX_MESSAGE_BYTES + 1]).is_err());
    let valid = witness();
    valid.validate().unwrap();
    for journal in [false, true] {
        for bytes in [Vec::new(), vec![0; MAX_WITNESS_BYTES + 1], vec![0xff]] {
            let mut changed = valid.clone();
            if journal {
                changed.original_journal_bytes = bytes;
            } else {
                changed.witness_bytes = bytes;
            }
            assert!(changed.validate().is_err());
        }
    }
}
#[test]
fn terminal_and_retained_physical_generation_policy_pin_are_checked_against_loaded_record() {
    for request in cases() {
        let retained = match &request.operation {
            Operation::VerifyOriginalWithObserver { retained, .. }
            | Operation::VerifyTerminalWithObserver { retained, .. } => retained,
            _ => unreachable!(),
        };
        let terminal = match &request.operation {
            Operation::VerifyTerminalWithObserver { terminal, .. } => Some(terminal.as_ref()),
            _ => None,
        };
        request.validate_recorded(retained, terminal).unwrap();
        for path in [
            "/generation",
            "/physical/storageId",
            "/physical/incarnationId",
            "/physical/journalId",
            "/physical/originalIdentityDigest",
            "/originalRecordDigest",
        ] {
            let mut changed = serde_json::to_value(retained).unwrap();
            *changed.pointer_mut(path).unwrap() = match path {
                "/generation" => json!(u64::MAX - 1),
                "/physical/originalIdentityDigest" | "/originalRecordDigest" => {
                    json!(support::hash(b"changed full policy pin identity"))
                }
                _ => json!("copied-store"),
            };
            let changed: Retained = serde_json::from_value(changed).unwrap();
            assert!(request.validate_recorded(&changed, terminal).is_err());
        }
        if let Some(terminal) = terminal {
            assert!(request.validate_recorded(retained, None).is_err());
            let mut changed = terminal.clone();
            changed.original_world_journal.push(0);
            assert!(request.validate_recorded(retained, Some(&changed)).is_err());
            let mut changed = terminal.clone();
            changed
                .acknowledgement
                .acknowledgement
                .kernel
                .runtime_instance_id = "other-runtime".into();
            assert!(request.validate_recorded(retained, Some(&changed)).is_err());
            let mut changed = terminal.clone();
            changed.original_world_journal.push(0);
            changed.original_world_journal_digest =
                ContentDigest::new(support::hash(&changed.original_world_journal)).unwrap();
            assert!(request.validate_recorded(retained, Some(&changed)).is_err());
        }
    }
}
#[test]
fn actual_published_0_1_6_all_old_verifier_requests_and_statuses_keep_named_bytes() {
    let rows: Value =
        serde_json::from_str(include_str!("fixtures/old0_1_6_verifier_wire.json")).unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 20);
    for row in rows.as_array().unwrap() {
        let bytes: Vec<u8> = serde_json::from_value(row["bytes"].clone()).unwrap();
        if row["kind"] == "request" {
            let value: Request = serde_json::from_value(row["value"].clone()).unwrap();
            assert_eq!(rmp_serde::to_vec_named(&value).unwrap(), bytes);
            assert_eq!(Request::decode_named(&bytes).unwrap(), value);
        } else {
            let value: Reply = serde_json::from_value(row["value"].clone()).unwrap();
            assert_eq!(rmp_serde::to_vec_named(&value).unwrap(), bytes);
            assert_eq!(Reply::decode_named(&bytes).unwrap(), value);
        }
    }
}
#[test]
fn source_selection_observer_uses_same_original_family_input_subject_and_readonly_fence() {
    use retention::{SubjectSourceInvocation, SubjectSourceOperation};
    let source: Value =
        serde_json::from_str(include_str!("fixtures/retention_source_original.json")).unwrap();
    let (mut retained, _) = support::retained();
    retained.original.admission =
        serde_json::from_value(source["sourceCommitAck"]["admission"].clone()).unwrap();
    let operation: SubjectSourceOperation =
        serde_json::from_value(source["selectPayload"].clone()).unwrap();
    let SubjectSourceOperation::SubjectSourceSelect(select) = &operation else {
        unreachable!()
    };
    retained.original.material.sha256 = select.source.content_digest.clone();
    retained.original.material.size_bytes = select.source.size_bytes;
    retained.original.content.digest =
        ContentDigest::new(select.source.content_digest.clone()).unwrap();
    retained.original.content.size_bytes = select.source.size_bytes;
    let context = "exact original source context (synthetic not authenticated)";
    let action = "exact original source action (synthetic not authenticated)";
    retained
        .original
        .admission
        .correlation
        .forwarded_context_proof_digest = support::hash(context.as_bytes());
    retained.original.admission.correlation.action_proof_digest = support::hash(action.as_bytes());
    retained.original.invocation = Invocation::SubjectSource(Box::new(SubjectSourceInvocation {
        schema_version: univers_aip_contracts_world_ipc::subject_source::SCHEMA.into(),
        operation,
        forwarded_security_context: context.into(),
        runtime_action_admission: action.into(),
    }));
    let observer = Invocation::SubjectSource(Box::new(SubjectSourceInvocation {
        schema_version: univers_aip_contracts_world_ipc::subject_source::SCHEMA.into(),
        operation: SubjectSourceOperation::SubjectSourceGetOrResume(
            serde_json::from_value(source["recoveryRequest"].clone()).unwrap(),
        ),
        forwarded_security_context: "fresh source observer context (synthetic)".into(),
        runtime_action_admission: "fresh source readonly action (synthetic)".into(),
    }));
    let request = Request::new(Operation::VerifyOriginalWithObserver {
        retained: Box::new(retained),
        observer,
    });
    request.validate().unwrap();
    assert_eq!(
        Request::decode_named(&rmp_serde::to_vec_named(&request).unwrap()).unwrap(),
        request
    );
    let mut changed = request.clone();
    let Operation::VerifyOriginalWithObserver {
        observer: Invocation::SubjectSource(value),
        ..
    } = &mut changed.operation
    else {
        unreachable!()
    };
    let SubjectSourceOperation::SubjectSourceGetOrResume(read) = &mut value.operation else {
        unreachable!()
    };
    read.subject_id = "other-subject".into();
    assert!(changed.validate().is_err());
    let mut different = request;
    let Operation::VerifyOriginalWithObserver { observer, .. } = &mut different.operation else {
        unreachable!()
    };
    let Operation::VerifyOriginalWithObserver {
        observer: spatial, ..
    } = cases().remove(0).operation
    else {
        unreachable!()
    };
    *observer = spatial;
    assert!(different.validate().is_err());
}
