#![cfg(feature = "artifact-physical-effects")]
use univers_aip_contracts_data_effects::artifact_effects::*;
use univers_aip_contracts_file_authority_ipc::content::physical_effects as wire;
fn query() -> ArtifactPhysicalEffectQuery {
    ArtifactPhysicalEffectQuery::from_json(include_bytes!("fixtures/physical_query.json")).unwrap()
}
fn observation(state: ArtifactPhysicalEffectState) -> wire::Reply {
    let query = query();
    wire::Reply {
        schema: wire::SCHEMA.into(),
        query: query.clone(),
        outcome: wire::Outcome::Observation {
            observation: Box::new(ArtifactPhysicalEffectObservation { query, state }),
        },
    }
}
#[test]
fn unrecorded_pending_staging_and_ambiguity_are_distinct_from_publication() {
    for state in [
        ArtifactPhysicalEffectState::NotRecorded,
        ArtifactPhysicalEffectState::Pending,
        ArtifactPhysicalEffectState::Staged {
            upload_id: ArtifactEffectId::new("upload-1").unwrap(),
        },
        ArtifactPhysicalEffectState::Ambiguous {
            reason: ArtifactEffectReason::PublicationAmbiguous,
        },
    ] {
        let reply = observation(state);
        let bytes = reply.to_json(&query()).unwrap();
        assert_eq!(wire::Reply::from_json(&bytes, &query()).unwrap(), reply);
        assert!(!String::from_utf8(bytes).unwrap().contains("complete"));
    }
}
#[test]
fn original_created_receipt_keeps_max_u64_and_protected_byte_distinctions() {
    let query = query();
    let original = ArtifactPhysicalEffectObservation::from_json(
        include_bytes!("fixtures/physical_published.json"),
        &query,
    )
    .unwrap();
    let reply = observation(original.state.clone());
    let bytes = reply.to_json(&query).unwrap();
    let decoded = wire::Reply::from_json(&bytes, &query).unwrap();
    assert_eq!(reply, decoded);
    if let ArtifactPhysicalEffectState::Published { receipt } = original.state {
        assert_eq!(receipt.outcome, ArtifactPhysicalOutcome::Created);
        assert_eq!(receipt.bytes_written.get(), u64::MAX);
        assert_eq!(receipt.logical_size_bytes.get(), 9007199254740993);
        assert_ne!(receipt.logical_sha256, receipt.physical_sha256);
        let mut replay = receipt.clone();
        replay.outcome = ArtifactPhysicalOutcome::Reused;
        replay.bytes_written = ArtifactEffectCount::new(0);
        assert!(replay.validate_replay_of(&receipt).is_err());
    } else {
        panic!("expected original publication");
    }
}
#[test]
fn reply_rejects_changed_original_scope_write_intent_or_content_lineage() {
    let reply = observation(ArtifactPhysicalEffectState::NotRecorded);
    let mut changed = query();
    changed.binding.attempt.scope.organization_id = ArtifactEffectId::new("other-org").unwrap();
    assert!(reply.validate_for(&changed).is_err());
    let mut changed = query();
    changed.binding.attempt.scope.world_id = ArtifactEffectId::new("other-world").unwrap();
    assert!(reply.validate_for(&changed).is_err());
    let mut changed = query();
    changed.binding.attempt.attempt_id = ArtifactEffectId::new("other-attempt").unwrap();
    assert!(reply.validate_for(&changed).is_err());
    let mut changed = query();
    changed.binding.write_id = ArtifactEffectId::new("other-write").unwrap();
    assert!(reply.validate_for(&changed).is_err());
    let mut changed = query();
    changed.intent_digest =
        ArtifactEffectDigest::new(format!("sha256:{}", "c".repeat(64))).unwrap();
    assert!(reply.validate_for(&changed).is_err());
    let mut changed = query();
    changed.expected_authority.generation =
        ArtifactEffectId::new("copied-content-lineage").unwrap();
    assert!(reply.validate_for(&changed).is_err());
}
#[test]
fn unsupported_is_explicit_and_full_selector_is_retained() {
    let request = query();
    let reply = wire::Reply {
        schema: wire::SCHEMA.into(),
        query: request.clone(),
        outcome: wire::Outcome::Error {
            error: wire::Error {
                code: wire::ErrorCode::Unsupported,
                reason: None,
            },
        },
    };
    let bytes = reply.to_json(&request).unwrap();
    assert_eq!(wire::Reply::from_json(&bytes, &request).unwrap(), reply);
    assert!(wire::Reply::from_json(&vec![b' '; wire::MAX_RESPONSE_BYTES + 1], &request).is_err());
    let mut malformed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    malformed["complete"] = serde_json::json!(true);
    assert!(wire::Reply::from_json(&serde_json::to_vec(&malformed).unwrap(), &request).is_err());
    let mut trailing = bytes.clone();
    trailing.push(b'0');
    assert!(wire::Reply::from_json(&trailing, &request).is_err());
}

#[test]
fn request_codec_preserves_data_json_and_rejects_unknown_protocol_and_fields() {
    let request = query();
    let encoded = wire::encode_request(&request).unwrap();
    assert_eq!(wire::decode_request(&encoded).unwrap(), request);
    assert_eq!(encoded, request.to_json().unwrap());
    for malformed in [
        serde_json::json!({"version": 2}),
        serde_json::json!({"schema": "old-content"}),
        serde_json::json!({"actor": "forged"}),
    ] {
        let mut value = serde_json::to_value(&request).unwrap();
        for (key, value_to_set) in malformed.as_object().unwrap() {
            value[key] = value_to_set.clone();
        }
        assert!(wire::decode_request(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    let mut value = serde_json::to_value(&request).unwrap();
    value["binding"]["attempt"]["scope"]["permission"] = serde_json::json!(true);
    assert!(wire::decode_request(&serde_json::to_vec(&value).unwrap()).is_err());
    let mut trailing = encoded.clone();
    trailing.push(b'0');
    assert!(wire::decode_request(&trailing).is_err());
    assert!(matches!(
        wire::decode_request(&vec![b' '; wire::MAX_REQUEST_BYTES + 1]),
        Err(ArtifactEffectError::ResourceExhausted)
    ));
    let duplicate = String::from_utf8(encoded).unwrap().replacen(
        "\"version\":1",
        "\"version\":1,\"version\":1",
        1,
    );
    assert!(wire::decode_request(duplicate.as_bytes()).is_err());
}

#[test]
fn negotiation_and_errors_do_not_claim_query_support_or_authority() {
    for support in [wire::Support::Unsupported, wire::Support::Query] {
        let mut capabilities = wire::Capabilities {
            schema: wire::SCHEMA.into(),
            capability: ARTIFACT_PHYSICAL_EFFECT_CAPABILITY.into(),
            support,
        };
        capabilities.validate().unwrap();
        capabilities.schema = "legacy-content".into();
        assert!(matches!(
            capabilities.validate(),
            Err(ArtifactEffectError::Unsupported)
        ));
        capabilities.schema = wire::SCHEMA.into();
        capabilities.capability = "content-engine".into();
        assert!(matches!(
            capabilities.validate(),
            Err(ArtifactEffectError::Unsupported)
        ));
    }
    for code in [
        wire::ErrorCode::Invalid,
        wire::ErrorCode::Conflict,
        wire::ErrorCode::Unsupported,
        wire::ErrorCode::Denied,
        wire::ErrorCode::ResourceExhausted,
        wire::ErrorCode::SnapshotInvalidated,
        wire::ErrorCode::Unavailable,
    ] {
        let request = query();
        let mut reply = wire::Reply {
            schema: wire::SCHEMA.into(),
            query: request.clone(),
            outcome: wire::Outcome::Error {
                error: wire::Error { code, reason: None },
            },
        };
        let bytes = reply.to_json(&request).unwrap();
        assert_eq!(wire::Reply::from_json(&bytes, &request).unwrap(), reply);
        if let wire::Outcome::Error { error } = &mut reply.outcome {
            error.reason = Some("x".repeat(wire::MAX_ERROR_REASON_BYTES));
        }
        reply.validate_for(&request).unwrap();
        for reason in [
            "".into(),
            " padded".into(),
            "newline\n".into(),
            "é".repeat(wire::MAX_ERROR_REASON_BYTES / 2 + 1),
        ] {
            if let wire::Outcome::Error { error } = &mut reply.outcome {
                error.reason = Some(reason);
            }
            assert!(reply.to_json(&request).is_err());
        }
    }
}

#[test]
fn nested_publication_and_observation_must_match_the_exact_original() {
    let request = query();
    let original = ArtifactPhysicalEffectObservation::from_json(
        include_bytes!("fixtures/physical_published.json"),
        &request,
    )
    .unwrap();
    let reply = observation(original.state);
    let valid = serde_json::to_value(&reply).unwrap();
    for (pointer, replacement) in [
        ("/schema", serde_json::json!("unsupported")),
        (
            "/outcome/observation/query/binding/attempt/requestId",
            serde_json::json!("other-request"),
        ),
        (
            "/outcome/observation/state/receipt/binding/writeId",
            serde_json::json!("other-write"),
        ),
        (
            "/outcome/observation/state/receipt/intentDigest",
            serde_json::json!(format!("sha256:{}", "c".repeat(64))),
        ),
        (
            "/outcome/observation/state/receipt/authority/generation",
            serde_json::json!("other-generation"),
        ),
        (
            "/outcome/observation/state/receipt/physicalSizeBytes",
            serde_json::json!("1"),
        ),
        (
            "/outcome/observation/state/receipt/outcome",
            serde_json::json!("reused"),
        ),
        (
            "/outcome/observation/state/receipt/bytesWritten",
            serde_json::json!(18446744073709551615u64),
        ),
    ] {
        let mut malformed = valid.clone();
        *malformed.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            wire::Reply::from_json(&serde_json::to_vec(&malformed).unwrap(), &request).is_err(),
            "{pointer}"
        );
    }
    let mut malformed = valid;
    malformed["outcome"]["observation"]["state"]["receipt"]["trusted"] = serde_json::json!(true);
    assert!(wire::Reply::from_json(&serde_json::to_vec(&malformed).unwrap(), &request).is_err());
}

#[test]
fn every_selector_component_is_correlated_even_for_unknown_results() {
    let request = query();
    let reply = observation(ArtifactPhysicalEffectState::NotRecorded);
    for pointer in [
        "/binding/attempt/scope/invokingOwner",
        "/binding/attempt/scope/worldIncarnation/dataNamespace",
        "/binding/attempt/scope/worldIncarnation/generation",
        "/binding/attempt/requestId",
        "/expectedAuthority/authorityId",
    ] {
        let mut changed = serde_json::to_value(&request).unwrap();
        *changed.pointer_mut(pointer).unwrap() = serde_json::json!("other");
        let changed = wire::decode_request(&serde_json::to_vec(&changed).unwrap()).unwrap();
        assert!(
            matches!(
                reply.validate_for(&changed),
                Err(ArtifactEffectError::Conflict(_))
            ),
            "{pointer}"
        );
    }
}
