use serde_json::json;
use univers_aip_contracts_data::storage::file_store::{
    FileContentReconciliationIssue, FileContentReconciliationMode, FileContentReconciliationReport,
};
use univers_aip_contracts_file_authority_ipc::{Reply, Request};

#[test]
fn mode_and_limit_reach_world_unchanged_for_owner_validation() {
    for mode in [
        FileContentReconciliationMode::Audit,
        FileContentReconciliationMode::Repair,
    ] {
        for limit in [0, 1, 256, 10_000, 10_001] {
            let request = Request::ReconcilePendingContentWrites { mode, limit };
            let expected = json!({"ReconcilePendingContentWrites": {"mode": mode, "limit": limit}});
            assert_eq!(serde_json::to_value(&request).unwrap(), expected);
            for bytes in [
                rmp_serde::to_vec(&request).unwrap(),
                rmp_serde::to_vec_named(&request).unwrap(),
            ] {
                let decoded: Request = rmp_serde::from_slice(&bytes).unwrap();
                assert_eq!(serde_json::to_value(decoded).unwrap(), expected);
            }
        }
    }
}

#[test]
fn report_preserves_partial_failures_and_does_not_collapse_audit_into_repair() {
    let report = FileContentReconciliationReport {
        mode: FileContentReconciliationMode::Audit,
        scanned: 7,
        finalized: 0,
        already_applied: 1,
        ready_to_finalize: 2,
        missing_content: 1,
        abandoned_missing_content: 0,
        pending_conflicts: 2,
        failures: vec![FileContentReconciliationIssue {
            intent_id: "intent-7".into(),
            path: "files/report".into(),
            digest: "digest".into(),
            message: "content unavailable".into(),
        }],
    };
    let reply = Reply::ContentWritesReconciled(report.clone());
    let json = serde_json::to_value(&reply).unwrap();
    assert_eq!(json["ContentWritesReconciled"]["readyToFinalize"], 2);
    assert_eq!(
        json["ContentWritesReconciled"]["failures"][0]["intentId"],
        "intent-7"
    );
    let decoded: Reply = rmp_serde::from_slice(&rmp_serde::to_vec_named(&reply).unwrap()).unwrap();
    match decoded {
        Reply::ContentWritesReconciled(actual) => assert_eq!(actual, report),
        other => panic!("unexpected reply: {other:?}"),
    }
}
