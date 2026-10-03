#![cfg(feature = "world-verifier")]
use serde_json::json;
use univers_aip_contracts_data::content_types::ContentDigest;
use univers_aip_contracts_file_authority_ipc::{
    content::retention::PhysicalStore, content_world_verifier::*,
};
#[path = "fixtures/retention_support.rs"]
mod support;
fn witness() -> FullWitness {
    // Raw private storage encoding: not JSON, includes complete max-u64/binary tails.
    let bytes = vec![
        0xc1, 0xff, 0, 0x91, 0xcf, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    ];
    let journal = vec![0xff, 0, 0x80, 0xc1, 0, 0x7f, 0xfe];
    FullWitness {
        witness_digest: ContentDigest::new(support::hash(&bytes)).unwrap(),
        witness_bytes: bytes,
        original_journal_digest: ContentDigest::new(support::hash(&journal)).unwrap(),
        original_journal_bytes: journal,
    }
}
fn cases() -> Vec<Request> {
    let (mut retained, _) = support::retained();
    retained.generation = u64::MAX;
    let inspection = support::status(&retained);
    let univers_aip_contracts_file_authority_ipc::content::retention::Request::Status {
        observer,
        ..
    } = inspection
    else {
        unreachable!()
    };
    vec![
        Request::new(Operation::VerifyAcquisition(Box::new(
            retained.original.clone(),
        ))),
        Request::new(Operation::VerifyOriginal(Box::new(retained.clone()))),
        Request::new(Operation::InspectOriginal {
            retained: Box::new(retained.clone()),
            observer,
        }),
        Request::new(Operation::VerifyTerminal {
            retained: Box::new(retained.clone()),
            terminal: Box::new(support::ack(&retained, false)),
        }),
    ]
}
#[test]
fn complete_original_observer_retained_terminal_and_opaque_evidence_roundtrip() {
    assert_eq!(PROVIDER, "world");
    assert_eq!(DIRECTION, Direction::ContentToWorld);
    assert_ne!(
        ENDPOINT,
        univers_aip_contracts_file_authority_ipc::content::ENDPOINT
    );
    for request in cases() {
        request.validate().unwrap();
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
            let reply = Reply {
                request: request.clone(),
                status: Status::UntrustedWitness(Box::new(witness())),
            };
            reply.validate_for(&request).unwrap();
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
            let Status::UntrustedWitness(w) = decoded.status else {
                panic!("wire must remain untrusted")
            };
            assert_eq!(*w, witness());
        }
        for status in [
            Status::Denied {
                reason: "current read policy revoked".into(),
            },
            Status::Unavailable {
                reason: "original copied or missing".into(),
            },
            Status::RecoveryRequired {
                reason: "queued writer uncertain; keep held".into(),
            },
        ] {
            let reply = Reply {
                request: request.clone(),
                status,
            };
            Reply::decode_named(&rmp_serde::to_vec_named(&reply).unwrap())
                .unwrap()
                .validate_for(&request)
                .unwrap();
        }
    }
}
#[test]
fn strict_schema_operation_unknown_duplicate_positional_trailing_malformed_bounds() {
    let request = &cases()[0];
    for (path, value) in [
        ("/schemaVersion", json!("other/v1")),
        ("/provider", json!("files")),
        ("/capability", json!("content-original-retention-v1")),
        ("/direction", json!("world_to_content")),
        ("/operation/operation", json!("verify_reference_removal")),
    ] {
        let mut candidate = serde_json::to_value(request).unwrap();
        *candidate.pointer_mut(path).unwrap() = value;
        assert!(Request::decode_named(&rmp_serde::to_vec_named(&candidate).unwrap()).is_err());
        assert!(Request::decode_json(&serde_json::to_vec(&candidate).unwrap()).is_err());
    }
    let mut nested = serde_json::to_value(request).unwrap();
    nested["operation"]["value"]["content"]["scope"]["callerInstalledAllow"] = json!(true);
    assert!(Request::decode_named(&rmp_serde::to_vec_named(&nested).unwrap()).is_err());
    assert!(Request::decode_json(&serde_json::to_vec(&nested).unwrap()).is_err());
    let duplicate = serde_json::to_string(request).unwrap().replace(
        "\"provider\":\"world\"",
        "\"provider\":\"world\",\"provider\":\"world\"",
    );
    assert!(Request::decode_json(duplicate.as_bytes()).is_err());
    let mut trailing = rmp_serde::to_vec_named(request).unwrap();
    trailing.push(0);
    assert!(Request::decode_named(&trailing).is_err());
    assert!(Request::decode_named(&rmp_serde::to_vec(request).unwrap()).is_err());
    assert!(Request::decode_named(&[0xc1]).is_err());
    assert!(Request::decode_named(&vec![0; MAX_MESSAGE_BYTES + 1]).is_err());
    let reply = Reply {
        request: request.clone(),
        status: Status::UntrustedWitness(Box::new(witness())),
    };
    let mut nested = serde_json::to_value(&reply).unwrap();
    nested["status"]["value"]["verifiedAllow"] = json!(true);
    assert!(Reply::decode_named(&rmp_serde::to_vec_named(&nested).unwrap()).is_err());
    let mut trailing = rmp_serde::to_vec_named(&reply).unwrap();
    trailing.push(0);
    assert!(Reply::decode_named(&trailing).is_err());
    assert!(Reply::decode_named(&rmp_serde::to_vec(&reply).unwrap()).is_err());
}
#[test]
fn full_original_tamper_never_becomes_a_correlated_reply() {
    let request = cases().remove(0);
    for path in [
        "/invocation/value/forwardedSecurityContext",
        "/invocation/value/runtimeActionAdmission",
        "/admission/correlation/securityContext/principalId",
        "/admission/correlation/securityContext/credentialId",
        "/admission/correlation/securityContext/delegationChain/0/principalId",
        "/admission/correlation/actionId",
        "/admission/correlation/providerId",
        "/worldRuntime/runtimeInstanceId",
        "/material/artifactId",
    ] {
        let Operation::VerifyAcquisition(original) = &request.operation else {
            unreachable!()
        };
        let mut candidate = serde_json::to_value(original).unwrap();
        *candidate.pointer_mut(path).unwrap() = json!("changed-original");
        let changed: Result<Original, _> = serde_json::from_value(candidate);
        if let Ok(original) = changed {
            let altered = Request::new(Operation::VerifyAcquisition(Box::new(original)));
            let reply = Reply {
                request: altered,
                status: Status::Denied {
                    reason: "untrusted altered original".into(),
                },
            };
            assert!(reply.validate_for(&request).is_err());
        }
    }
    let original = match &request.operation {
        Operation::VerifyAcquisition(v) => v,
        _ => unreachable!(),
    };
    for (path, value) in [
        (
            "/admission/correlation/selectedWorldFence/revision",
            json!(13),
        ),
        ("/content/sizeBytes", json!(0)),
        ("/content/scope/worldInstanceId", json!("other-world")),
        ("/material/sha256", json!(support::hash(b"new material"))),
    ] {
        let mut candidate = serde_json::to_value(original).unwrap();
        *candidate.pointer_mut(path).unwrap() = value;
        let original: Original = serde_json::from_value(candidate).unwrap();
        assert!(
            Request::new(Operation::VerifyAcquisition(Box::new(original)))
                .validate()
                .is_err()
        );
    }
}
#[test]
fn exact_recorded_store_generation_pin_and_policy_binding_are_required() {
    let (retained, _) = support::retained();
    let request = Request::new(Operation::VerifyOriginal(Box::new(retained.clone())));
    request.validate_recorded(&retained, None).unwrap();
    let mut changes = vec![];
    let mut v = retained.clone();
    v.generation += 1;
    changes.push(v);
    let mut v = retained.clone();
    v.physical.incarnation_id = "copied-original-store".into();
    changes.push(v);
    let mut v = retained.clone();
    v.physical.journal_id = "replacement-journal".into();
    changes.push(v);
    let mut v = retained.clone();
    v.original_record_digest = ContentDigest::new(support::hash(b"changed pin policy")).unwrap();
    changes.push(v);
    for changed in changes {
        assert!(request.validate_recorded(&changed, None).is_err());
    }
    let PhysicalStore {
        original_identity_digest,
        ..
    } = &retained.physical;
    assert!(!original_identity_digest.as_str().is_empty());
    let failure = Reply {
        request: request.clone(),
        status: Status::Unavailable {
            reason: "actual original store corrupt or missing".into(),
        },
    };
    failure.validate_for(&request).unwrap();
    assert!(matches!(failure.status, Status::Unavailable { .. }));
}
#[test]
fn expired_original_management_and_fresh_readonly_observer_do_not_reacquire_or_write() {
    let requests = cases();
    let acquire = &requests[0];
    let Operation::VerifyAcquisition(original) = &acquire.operation else {
        unreachable!()
    };
    let expiry = original.admission.not_after_unix_ms;
    acquire.validate_acquisition_at(expiry - 1).unwrap();
    assert!(acquire.validate_acquisition_at(expiry).is_err());
    requests[1].validate().unwrap();
    assert!(requests[1].validate_acquisition_at(expiry - 1).is_err());
    let mut inspect = requests[2].clone();
    inspect.validate().unwrap();
    assert!(inspect.validate_acquisition_at(expiry - 1).is_err());
    let Operation::InspectOriginal {
        observer: Invocation::Spatial(value),
        ..
    } = &mut inspect.operation
    else {
        unreachable!()
    };
    let univers_aip_contracts_world_ipc::spatial_mutation::SpatialBindingOperation::GetOrResume(r) =
        &mut value.operation
    else {
        unreachable!()
    };
    r.read_only = Some(false);
    assert!(inspect.validate().is_err());
    let mut changed = requests[2].clone();
    let Operation::InspectOriginal {
        observer: Invocation::Spatial(value),
        ..
    } = &mut changed.operation
    else {
        unreachable!()
    };
    value.runtime_action_admission = "different FRESH observer proof".into();
    changed.validate().unwrap();
    let reply = Reply {
        request: changed,
        status: Status::UntrustedWitness(Box::new(witness())),
    };
    assert!(reply.validate_for(&requests[2]).is_err());
    // Revocation is an actual World decision; wire carries denial, never makes it verified.
    let denied = Reply {
        request: requests[2].clone(),
        status: Status::Denied {
            reason: "fresh observer read policy revoked".into(),
        },
    };
    denied.validate_for(&requests[2]).unwrap();
    assert!(matches!(denied.status, Status::Denied { .. }));
}
#[test]
fn terminal_matches_actual_complete_journal_and_old_ack_does_not_release_successor() {
    let (retained, _) = support::retained();
    let terminal = support::ack(&retained, false);
    let request = Request::new(Operation::VerifyTerminal {
        retained: Box::new(retained.clone()),
        terminal: Box::new(terminal.clone()),
    });
    request
        .validate_recorded(&retained, Some(&terminal))
        .unwrap();
    assert!(request.validate_recorded(&retained, None).is_err());
    let mut successor = retained.clone();
    successor.generation += 1;
    assert!(request
        .validate_recorded(&successor, Some(&terminal))
        .is_err());
    let mut changed = terminal.clone();
    *changed.original_world_journal.last_mut().unwrap() ^= 1;
    assert!(request
        .validate_recorded(&retained, Some(&changed))
        .is_err());
    let mut changed = terminal;
    changed
        .acknowledgement
        .acknowledgement
        .kernel
        .runtime_instance_id = "old-runtime".into();
    assert!(request
        .validate_recorded(&retained, Some(&changed))
        .is_err());
    let uncertain = Reply {
        request,
        status: Status::RecoveryRequired {
            reason: "owned and queued terminal truth uncertain; keep held".into(),
        },
    };
    uncertain.validate().unwrap();
    assert!(matches!(uncertain.status, Status::RecoveryRequired { .. }));
}
#[test]
fn independent_full_witness_and_journal_bounds_digests_and_binary_tail_are_checked() {
    let valid = witness();
    valid.validate().unwrap();
    for journal in [false, true] {
        for bytes in [Vec::new(), vec![0; MAX_WITNESS_BYTES + 1], vec![0; 1]] {
            let mut changed = valid.clone();
            if journal {
                changed.original_journal_bytes = bytes;
            } else {
                changed.witness_bytes = bytes;
            }
            assert!(changed.validate().is_err());
        }
        let mut changed = valid.clone();
        if journal {
            *changed.original_journal_bytes.last_mut().unwrap() ^= 1;
        } else {
            *changed.witness_bytes.last_mut().unwrap() ^= 1;
        }
        assert!(changed.validate().is_err());
    }
    let request = cases().remove(0);
    let reply = Reply {
        request: request.clone(),
        status: Status::UntrustedWitness(Box::new(valid)),
    };
    let mut different = cases().remove(1);
    assert!(reply.validate_for(&different).is_err());
    different.capability = "different capability".into();
    assert!(reply.validate_for(&different).is_err());
}
