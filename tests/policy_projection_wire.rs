#![cfg(feature = "world-policy-projection")]
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use univers_aip_contracts_data::{
    content_types::ContentDigest,
    storage::{KvStore, KvValueCondition},
};
use univers_aip_contracts_file_authority_ipc::{
    content_world_policy_projection::*, content_world_verifier as verifier,
};
#[path = "fixtures/policy_projection_support.rs"]
mod support;
fn digest(bytes: &[u8]) -> ContentDigest {
    ContentDigest::new(format!("sha256:{:x}", Sha256::digest(bytes))).unwrap()
}
#[tokio::test]
async fn actual_issued_producer_and_decoder_full_snapshot_whole_original_readonly_terminal_roundtrip(
) {
    let f = support::Fixture::new(0).await;
    let value = f.projection().await;
    let rows: Vec<KvValueCondition> =
        rmp_serde::from_slice(&value.files_held_policy_snapshot).unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[1].expected, rows[2].expected);
    let before = f.kv.scan_prefix("").await.unwrap();
    for request in f.requests() {
        request.validate().unwrap();
        let reply = Reply {
            request: request.clone(),
            status: Status::UntrustedProjection(Box::new(value.clone())),
        };
        for named in [true, false] {
            let bytes = if named {
                rmp_serde::to_vec_named(&reply).unwrap()
            } else {
                serde_json::to_vec(&reply).unwrap()
            };
            let received = if named {
                Reply::decode_named(&bytes)
            } else {
                Reply::decode_json(&bytes)
            }
            .unwrap();
            assert_eq!(received, reply);
            assert_eq!(
                received
                    .validate_policy_for(&request, &f.selected, &f.access)
                    .unwrap(),
                1
            );
            let Status::UntrustedProjection(projected) = received.status else {
                panic!()
            };
            assert_eq!(
                projected.files_held_policy_snapshot,
                value.files_held_policy_snapshot
            );
            assert_eq!(
                projected.witness.original_journal_bytes,
                f.terminal.original_world_journal
            );
        }
    }
    assert_eq!(before, f.kv.scan_prefix("").await.unwrap());
}
#[tokio::test]
async fn full_raw_owner_rows_order_epoch_store_original_tamper_are_denied_by_issued_decoder() {
    let f = support::Fixture::new(0).await;
    let value = f.projection().await;
    let rows: Vec<KvValueCondition> =
        rmp_serde::from_slice(&value.files_held_policy_snapshot).unwrap();
    for kind in 0..5 {
        let mut rows = rows.clone();
        match kind {
            0 => rows.swap(0, 1),
            1 => {
                rows.pop();
            }
            2 => rows[2].expected = None,
            3 => rows.push(rows[0].clone()),
            _ => rows[0].expected.as_mut().unwrap().push(0),
        }
        let mut changed = value.clone();
        changed.files_held_policy_snapshot = rmp_serde::to_vec_named(&rows).unwrap();
        changed.files_held_policy_snapshot_digest = digest(&changed.files_held_policy_snapshot);
        changed.validate().unwrap();
        assert!(changed.validate_policy_for(&f.selected, &f.access).is_err());
    }
    let mut changed = value.clone();
    changed.files_held_policy_epoch = u64::MAX;
    changed.validate().unwrap();
    assert!(changed.validate_policy_for(&f.selected, &f.access).is_err());
    changed = value.clone();
    changed
        .original_operating_store_identity
        .push_str("-successor");
    assert!(changed.validate_policy_for(&f.selected, &f.access).is_err());
    for index in 0..5 {
        let mut original = f.access.clone();
        match index {
            0 => original.original_admission.push(0),
            1 => original.original_context.push(0),
            2 => original.credential.push('x'),
            3 => original.delegation = None,
            _ => original.principal.push('x'),
        };
        assert!(value.validate_policy_for(&f.selected, &original).is_err());
    }
}
#[tokio::test]
async fn saved_snapshot_after_actual_release_is_historical_not_current_permission() {
    let f = support::Fixture::new(0).await;
    let saved = f.projection().await;
    f.authority
        .release_after_terminal(&f.access, b"unsigned-fixture-terminal")
        .await
        .unwrap();
    assert!(f.authority.held_policy_snapshot(&f.access).await.is_err());
    assert_eq!(
        saved.validate_policy_for(&f.selected, &f.access).unwrap(),
        1
    );
    let mut rows: Vec<KvValueCondition> =
        rmp_serde::from_slice(&saved.files_held_policy_snapshot).unwrap();
    for row in &mut rows {
        row.expected = f.kv.get(&row.key).await.unwrap();
    }
    let mut released = saved;
    released.files_held_policy_snapshot = rmp_serde::to_vec_named(&rows).unwrap();
    released.files_held_policy_snapshot_digest = digest(&released.files_held_policy_snapshot);
    assert!(released
        .validate_policy_for(&f.selected, &f.access)
        .is_err());
}
#[tokio::test]
async fn full_message_profile_preserves_large_actual_issued_snapshot_without_truncation() {
    let f = support::Fixture::new(384 * 1024).await;
    let value = f.projection().await;
    assert!(value.files_held_policy_snapshot.len() <= MAX_SNAPSHOT_BYTES);
    let request = f.requests().remove(1);
    let reply = Reply {
        request: request.clone(),
        status: Status::UntrustedProjection(Box::new(value.clone())),
    };
    let bytes = serde_json::to_vec(&reply).unwrap();
    assert!(bytes.len() > verifier::MAX_MESSAGE_BYTES && bytes.len() < MAX_REPLY_BYTES);
    let received = Reply::decode_json(&bytes).unwrap();
    assert_eq!(received, reply);
    assert_eq!(
        received
            .validate_policy_for(&request, &f.selected, &f.access)
            .unwrap(),
        1
    );
    assert!(verifier::Reply::decode_json(&bytes).is_err());
}
#[tokio::test]
async fn no_missing_projection_legacy_expired_fallback_or_current_readonly_input_tamper() {
    let f = support::Fixture::new(0).await;
    let value = f.projection().await;
    let mut denied_caller = f.access.clone();
    denied_caller.principal = "unsigned-fixture-denied-caller".into();
    assert!(f
        .authority
        .held_policy_snapshot(&denied_caller)
        .await
        .is_err());
    let requests = f.requests();
    assert!(requests[0]
        .validate_acquisition_at(f.retained.original.admission.not_after_unix_ms)
        .is_err());
    assert!(
        Request::new(verifier::Request::new(verifier::Operation::VerifyOriginal(
            Box::new(f.retained.clone())
        )))
        .validate()
        .is_err()
    );
    assert!(Request::new(verifier::Request::new(
        verifier::Operation::VerifyTerminal {
            retained: Box::new(f.retained.clone()),
            terminal: Box::new(f.terminal.clone())
        }
    ))
    .validate()
    .is_err());
    for request in requests {
        let reply = Reply {
            request: request.clone(),
            status: Status::UntrustedProjection(Box::new(value.clone())),
        };
        let mut missing = serde_json::to_value(&reply).unwrap();
        missing["status"]["value"]
            .as_object_mut()
            .unwrap()
            .remove("filesHeldPolicySnapshot");
        assert!(Reply::decode_json(&serde_json::to_vec(&missing).unwrap()).is_err());
        for status in [
            Status::Denied {
                reason: "current Auth denied".into(),
            },
            Status::Unavailable {
                reason: "unsupported profile/same-store unavailable".into(),
            },
            Status::RecoveryRequired {
                reason: "owned plus queued terminal uncertain".into(),
            },
        ] {
            let reply = Reply {
                request: request.clone(),
                status,
            };
            assert!(reply
                .validate_policy_for(&request, &f.selected, &f.access)
                .is_err());
        }
    }
    let request = f.requests().remove(2);
    for (path, change) in [
        (
            "/verification/operation/value/observer/value/forwardedSecurityContext",
            json!("changed current credential"),
        ),
        (
            "/verification/operation/value/observer/value/operation/request/readOnly",
            json!(false),
        ),
        (
            "/verification/operation/value/observer/value/operation/request/expectedWorldRevision",
            json!(13),
        ),
        (
            "/verification/operation/value/retained/generation",
            json!(2),
        ),
    ] {
        let mut altered = serde_json::to_value(&request).unwrap();
        *altered.pointer_mut(path).unwrap() = change;
        if let Ok(changed) = serde_json::from_value::<Request>(altered) {
            let reply = Reply {
                request: changed,
                status: Status::UntrustedProjection(Box::new(value.clone())),
            };
            assert!(reply.validate_for(&request).is_err());
        }
    }
    let mut changed = value;
    changed.witness.original_journal_bytes.push(0);
    changed.witness.original_journal_digest = digest(&changed.witness.original_journal_bytes);
    assert!(Reply {
        request,
        status: Status::UntrustedProjection(Box::new(changed))
    }
    .validate()
    .is_err());
}
#[tokio::test]
async fn strict_profiles_snapshot_witness_bounds_duplicate_unknown_trailing_and_lossless_epoch() {
    use serde::{ser::SerializeMap, Serialize};
    struct Duplicate<'a>(&'a Request);
    impl Serialize for Duplicate<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let v = serde_json::to_value(self.0).unwrap();
            let mut m = serializer.serialize_map(Some(6))?;
            for (k, v) in v.as_object().unwrap() {
                m.serialize_entry(k, v)?;
            }
            m.serialize_entry("verification", &v["verification"])?;
            m.end()
        }
    }
    let f = support::Fixture::new(0).await;
    let request = f.requests().remove(1);
    assert!(
        Request::decode_named(&rmp_serde::to_vec_named(&Duplicate(&request)).unwrap()).is_err()
    );
    let duplicate = serde_json::to_string(&request).unwrap().replace(
        "\"verification\":",
        "\"verification\":null,\"verification\":",
    );
    assert!(Request::decode_json(duplicate.as_bytes()).is_err());
    let mut unknown = serde_json::to_value(&request).unwrap();
    unknown["verification"]["operation"]["value"]["observer"]["currentAllow"] = json!(true);
    assert!(Request::decode_json(&serde_json::to_vec(&unknown).unwrap()).is_err());
    let mut trailing = rmp_serde::to_vec_named(&request).unwrap();
    trailing.push(0);
    assert!(Request::decode_named(&trailing).is_err());
    assert!(Request::decode_named(&vec![0; MAX_REQUEST_BYTES + 1]).is_err());
    assert!(Reply::decode_named(&vec![0; MAX_REPLY_BYTES + 1]).is_err());
    let value = f.projection().await;
    for bytes in [vec![], vec![0; MAX_SNAPSHOT_BYTES + 1], vec![0xff]] {
        let mut changed = value.clone();
        changed.files_held_policy_snapshot = bytes;
        assert!(changed.validate().is_err());
    }
    let mut maximum = value;
    maximum.files_held_policy_epoch = u64::MAX;
    let bytes = rmp_serde::to_vec_named(&maximum).unwrap();
    let decoded: FullPolicyProjection = rmp_serde::from_slice(&bytes).unwrap();
    assert_eq!(decoded.files_held_policy_epoch, u64::MAX);
    assert!(decoded.validate_policy_for(&f.selected, &f.access).is_err());
}
#[test]
fn actual_published_0_1_7_thirty_requests_and_statuses_keep_exact_wire() {
    let rows: Vec<Value> =
        serde_json::from_str(include_str!("fixtures/old0_1_7_verifier_wire.json")).unwrap();
    assert_eq!(rows.len(), 30);
    for row in rows {
        let bytes: Vec<u8> = serde_json::from_value(row["bytes"].clone()).unwrap();
        if row["kind"] == "request" {
            let value: verifier::Request = serde_json::from_value(row["value"].clone()).unwrap();
            assert_eq!(rmp_serde::to_vec_named(&value).unwrap(), bytes);
            assert_eq!(verifier::Request::decode_named(&bytes).unwrap(), value);
        } else {
            let value: verifier::Reply = serde_json::from_value(row["value"].clone()).unwrap();
            assert_eq!(rmp_serde::to_vec_named(&value).unwrap(), bytes);
            assert_eq!(verifier::Reply::decode_named(&bytes).unwrap(), value);
        }
    }
}
