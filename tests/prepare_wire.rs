use serde_json::json;
use univers_aip_contracts_file_authority_ipc::{
    FileContentWritePrepareMode, FileContentWritePrepareOutcome, Reply, Request,
};

fn request() -> serde_json::Value {
    json!({"PrepareContentWrite": {
        "path": "documents/report.txt", "digest": format!("sha256:{}", "a".repeat(64)), "sizeBytes": 42,
        "mode": "create_only", "mutation": null
    }})
}

#[test]
fn preparation_preserves_conditions_across_json_and_both_messagepack_forms() {
    for mode in [
        json!("replace"),
        json!("create_only"),
        json!({"if_hash_matches": format!("sha256:{}", "b".repeat(64))}),
    ] {
        let mut value = request();
        value["PrepareContentWrite"]["mode"] = mode;
        let parsed: Request = serde_json::from_value(value.clone()).unwrap();
        for bytes in [
            rmp_serde::to_vec(&parsed).unwrap(),
            rmp_serde::to_vec_named(&parsed).unwrap(),
        ] {
            let decoded: Request = rmp_serde::from_slice(&bytes).unwrap();
            assert_eq!(serde_json::to_value(decoded).unwrap(), value);
        }
    }
}

#[test]
fn preparation_rejects_caller_supplied_authority_and_malformed_digest() {
    for field in [
        "scope",
        "intentId",
        "previous",
        "modifiedAtMs",
        "startedAtMs",
    ] {
        let mut value = request();
        value["PrepareContentWrite"][field] = json!(null);
        assert!(
            serde_json::from_value::<Request>(value).is_err(),
            "accepted {field}"
        );
    }
    let mut value = request();
    value["PrepareContentWrite"]["digest"] = json!("invalid");
    assert!(serde_json::from_value::<Request>(value).is_err());
}

#[test]
fn mutation_is_only_a_declared_link_and_false_condition_is_explicit() {
    let mut value = request();
    value["PrepareContentWrite"]["mutation"] = json!({
        "key": "caller-key", "operation": "write", "requestDigest": "caller-digest"
    });
    let Request::PrepareContentWrite(parsed) = serde_json::from_value(value).unwrap() else {
        panic!()
    };
    assert_eq!(parsed.mode, FileContentWritePrepareMode::CreateOnly);
    assert_eq!(parsed.mutation.unwrap().key, "caller-key");
    let reply = Reply::ContentWritePrepared(FileContentWritePrepareOutcome::ConditionNotMet);
    let bytes = rmp_serde::to_vec_named(&reply).unwrap();
    assert!(matches!(
        rmp_serde::from_slice::<Reply>(&bytes).unwrap(),
        Reply::ContentWritePrepared(FileContentWritePrepareOutcome::ConditionNotMet)
    ));
    assert_ne!(bytes, rmp_serde::to_vec_named(&Reply::Bool(false)).unwrap());
}
