use serde_json::json;
use univers_aip_contracts_data::file::FileCatalogRecord;
use univers_aip_contracts_file_authority_ipc::{Reply, Request};

#[test]
fn namespace_path_requests_reject_caller_authority_fields() {
    for operation in ["EnsureDirectory", "RemoveEmptyDirectory", "DeleteFile"] {
        let value = json!({operation: {"path": "documents/report"}});
        let parsed: Request = serde_json::from_value(value.clone()).unwrap();
        let bytes = rmp_serde::to_vec_named(&parsed).unwrap();
        assert_eq!(
            serde_json::to_value(rmp_serde::from_slice::<Request>(&bytes).unwrap()).unwrap(),
            value
        );
        for field in [
            "scope",
            "modifiedAtMs",
            "record",
            "previous",
            "expectedRevision",
        ] {
            let mut extra = value.clone();
            extra[operation][field] = json!(null);
            assert!(
                serde_json::from_value::<Request>(extra).is_err(),
                "accepted {operation}/{field}"
            );
        }
    }
}

#[test]
fn file_record_move_preserves_exact_paths_without_caller_records_or_time() {
    let value = json!({"MoveFileRecord": {"sourcePath":"records/a", "targetPath":"records/b"}});
    let request: Request = serde_json::from_value(value.clone()).unwrap();
    for bytes in [
        rmp_serde::to_vec(&request).unwrap(),
        rmp_serde::to_vec_named(&request).unwrap(),
    ] {
        let decoded: Request = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), value);
    }
    let mut extra = value;
    extra["MoveFileRecord"]["modifiedAtMs"] = json!(42);
    assert!(serde_json::from_value::<Request>(extra).is_err());
}

#[test]
fn authoritative_result_kinds_preserve_absence_and_no_change() {
    let record = FileCatalogRecord {
        path: "records/b".into(),
        digest: None,
        size_bytes: 0,
        is_directory: true,
        modified_at_ms: 123,
    };
    let mut file = record.clone();
    file.is_directory = false;
    let replies = [
        Reply::DirectoryEnsured {
            record,
            created: false,
        },
        Reply::DirectoryRemoved { removed: false },
        Reply::FileDeleted { deleted: false },
        Reply::FileRecordMoved {
            record: None,
            changed: false,
        },
        Reply::FileRecordMoved {
            record: Some(file.clone()),
            changed: false,
        },
        Reply::FileRecordMoved {
            record: Some(file),
            changed: true,
        },
    ];
    let values = replies
        .iter()
        .map(|reply| serde_json::to_value(reply).unwrap())
        .collect::<Vec<_>>();
    for (reply, expected) in replies.iter().zip(&values) {
        let bytes = rmp_serde::to_vec_named(reply).unwrap();
        let actual: Reply = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(serde_json::to_value(actual).unwrap(), *expected);
    }
    assert_ne!(values[3], values[4]);
    assert_ne!(values[4], values[5]);
    assert_ne!(values[1], values[2]);
}
