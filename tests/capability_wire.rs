#![cfg(feature = "capability")]
//! Frozen pre-migration enums prove both compact and named MessagePack compatibility.
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use univers_aip_contracts_data::file::*;
use univers_aip_contracts_file_authority_ipc::capability;
mod legacy {
    use super::*;
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Request {
        GetById {
            id: String,
            organization_id: String,
        },
        GetByPath {
            virtual_path: String,
            organization_id: String,
        },
        Exists {
            virtual_path: String,
            organization_id: String,
        },
        ReadBytes {
            id: String,
            organization_id: String,
        },
        ReadRange {
            id: String,
            organization_id: String,
            offset: u64,
            length: u64,
        },
        ReadText {
            id: String,
            organization_id: String,
        },
        WriteFile(FileWriteCommand),
        QueryFiles {
            organization_id: FileOrganizationId,
            query: FileListQuery,
        },
        // Legacy wire variants remain decode-only during the managed IPC migration.
        WriteBytes(WriteFileRequest),
        List {
            organization_id: String,
            options: ListOptions,
        },
        QueryPage {
            organization_id: FileOrganizationId,
            request: FilePageRequest,
        },
        SearchByName {
            organization_id: String,
            pattern: String,
            options: ListOptions,
        },
        Delete {
            id: String,
            organization_id: String,
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ManagedRequest {
        world_instance_id: String,
        request: Request,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum Reply {
        Metadata(FileMetadata),
        Exists(bool),
        Bytes(Bytes),
        Range(FileRangeRead),
        Text(String),
        Write(WriteFileResponse),
        List(ListResponse),
        Page(FilePage),
        Deleted,
        Error(FileCapabilityError),
    }
}
#[test]
fn selected_world_requests_preserve_all_variant_bytes() {
    use capability::Request::*;
    let id = || "file-1".to_string();
    let org = || "org-a".to_string();
    let organization = || FileOrganizationId::new("org-a").unwrap();
    let requests = vec![
        GetById {
            id: id(),
            organization_id: org(),
        },
        GetByPath {
            virtual_path: "/file".into(),
            organization_id: org(),
        },
        Exists {
            virtual_path: "/file".into(),
            organization_id: org(),
        },
        ReadBytes {
            id: id(),
            organization_id: org(),
        },
        ReadRange {
            id: id(),
            organization_id: org(),
            offset: 12,
            length: 5,
        },
        ReadText {
            id: id(),
            organization_id: org(),
        },
        WriteFile(FileWriteCommand {
            organization_id: organization(),
            location: FileWriteLocation::CategoryName {
                category: FileCategory::Documents,
                file_name: FileName::new("quota").unwrap(),
            },
            mime_type: Some("text/plain".into()),
            content: Bytes::from_static(b"x"),
            precondition: FileWritePrecondition::Any,
            skip_if_hash: None,
            idempotency_key: Some("receipt-1".into()),
            history: FileWriteHistoryPolicy::Preserve,
        }),
        QueryFiles {
            organization_id: organization(),
            query: FileListQuery::default(),
        },
        WriteBytes(WriteFileRequest {
            organization_id: org(),
            file_name: "legacy".into(),
            category: FileCategory::Documents,
            mime_type: "text/plain".into(),
            virtual_path: None,
            content: Bytes::from_static(b"x"),
            no_snapshot: false,
            if_not_hash: None,
            if_match: None,
            if_absent: false,
        }),
        List {
            organization_id: org(),
            options: ListOptions::default(),
        },
        QueryPage {
            organization_id: organization(),
            request: FilePageRequest {
                criteria: FilePageCriteria::default(),
                cursor: None,
                limit: 10,
            },
        },
        SearchByName {
            organization_id: org(),
            pattern: "*.csv".into(),
            options: ListOptions::default(),
        },
        Delete {
            id: id(),
            organization_id: org(),
        },
    ];
    assert_eq!(requests.len(), 13);
    for request in requests {
        let current = capability::ManagedRequest {
            world_instance_id: "world-a".into(),
            request,
        };
        let original: legacy::ManagedRequest =
            serde_json::from_value(serde_json::to_value(&current).unwrap()).unwrap();
        assert_eq!(
            rmp_serde::to_vec(&current).unwrap(),
            rmp_serde::to_vec(&original).unwrap()
        );
        assert_eq!(
            rmp_serde::to_vec_named(&current).unwrap(),
            rmp_serde::to_vec_named(&original).unwrap()
        );
        // The production transport uses named MessagePack. Some legacy C0
        // adjacently tagged payloads cannot decode compact arrays; preserve
        // that behavior rather than claiming a new compact wire guarantee.
        let compact = rmp_serde::to_vec(&original).unwrap();
        let old_compact = rmp_serde::from_slice::<legacy::ManagedRequest>(&compact)
            .map(|value| serde_json::to_value(value).unwrap())
            .map_err(|error| error.to_string());
        let new_compact = rmp_serde::from_slice::<capability::ManagedRequest>(&compact)
            .map(|value| serde_json::to_value(value).unwrap())
            .map_err(|error| error.to_string());
        assert_eq!(old_compact, new_compact);
        let decoded: capability::ManagedRequest =
            rmp_serde::from_slice(&rmp_serde::to_vec_named(&original).unwrap()).unwrap();
        assert_eq!(decoded.world_instance_id, "world-a");
    }
}
#[test]
fn replies_and_missing_world_preserve_protocol_behavior() {
    for reply in [
        capability::Reply::Exists(false),
        capability::Reply::Bytes(Bytes::from_static(b"raw bytes")),
        capability::Reply::Range(FileRangeRead {
            bytes: Bytes::from_static(b"abc"),
            offset: 12,
            total_size: 20,
            eof: false,
        }),
        capability::Reply::Text("text".into()),
        capability::Reply::Deleted,
        capability::Reply::Error(FileCapabilityError::CrossOrgDenied("file-1".into())),
    ] {
        let original: legacy::Reply =
            serde_json::from_value(serde_json::to_value(&reply).unwrap()).unwrap();
        assert_eq!(
            rmp_serde::to_vec(&reply).unwrap(),
            rmp_serde::to_vec(&original).unwrap()
        );
        assert_eq!(
            rmp_serde::to_vec_named(&reply).unwrap(),
            rmp_serde::to_vec_named(&original).unwrap()
        );
    }
    let missing =
        serde_json::json!({"request":{"Delete":{"id":"file-1","organization_id":"org-a"}}});
    assert!(serde_json::from_value::<capability::ManagedRequest>(missing.clone()).is_err());
    assert!(serde_json::from_value::<legacy::ManagedRequest>(missing).is_err());
}
