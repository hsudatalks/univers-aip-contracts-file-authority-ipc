use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use univers_aip_contracts_file_authority_ipc::content;
#[path = "fixtures/content_legacy.rs"]
mod legacy;
fn parity<Old: DeserializeOwned + Serialize, New: DeserializeOwned + Serialize>(value: Value) {
    let old: Old = serde_json::from_value(value.clone()).unwrap();
    let new: New = serde_json::from_value(value).unwrap();
    for (old_bytes, new_bytes) in [
        (
            rmp_serde::to_vec_named(&old).unwrap(),
            rmp_serde::to_vec_named(&new).unwrap(),
        ),
        (
            rmp_serde::to_vec(&old).unwrap(),
            rmp_serde::to_vec(&new).unwrap(),
        ),
    ] {
        assert_eq!(old_bytes, new_bytes);
        match (
            rmp_serde::from_slice::<Old>(&old_bytes),
            rmp_serde::from_slice::<New>(&old_bytes),
        ) {
            (Ok(old), Ok(new)) => assert_eq!(
                serde_json::to_value(old).unwrap(),
                serde_json::to_value(new).unwrap()
            ),
            (Err(old), Err(new)) => assert_eq!(old.to_string(), new.to_string()),
            _ => panic!("decoder behavior changed"),
        }
    }
}
#[test]
fn all_content_requests_preserve_legacy_bytes() {
    let fixtures: Value = serde_json::from_str(include_str!("fixtures/content_wire.json")).unwrap();
    let values = fixtures["requests"].as_array().unwrap();
    assert_eq!(values.len(), 10);
    for value in values {
        parity::<legacy::Request, content::Request>(value.clone());
    }
}
#[test]
fn all_content_replies_and_errors_preserve_legacy_bytes() {
    let fixtures: Value = serde_json::from_str(include_str!("fixtures/content_wire.json")).unwrap();
    let values = fixtures["replies"].as_array().unwrap();
    assert_eq!(values.len(), 15);
    for value in values {
        parity::<legacy::Reply, content::Reply>(value.clone());
    }
}
#[test]
fn malformed_digest_is_not_accepted_by_pure_wire() {
    let mut fixtures: Value =
        serde_json::from_str(include_str!("fixtures/content_wire.json")).unwrap();
    fixtures["requests"][6]["value"]["digest"] = Value::String("bad-digest".into());
    assert!(serde_json::from_value::<content::Request>(fixtures["requests"][6].clone()).is_err());
}
