use serde::Serialize;
use univers_aip_contracts_file_authority_ipc::{Reply, Request};
#[derive(Serialize)]
enum LegacyRequest {
    ScanRecordsPage {
        path: String,
        cursor: Option<String>,
        expected_revision: Option<u64>,
        limit: usize,
    },
}
#[test]
fn revision_fenced_page_is_byte_compatible_with_the_existing_named_wire() {
    let bytes = rmp_serde::to_vec_named(&LegacyRequest::ScanRecordsPage {
        path: "world/path".into(),
        cursor: Some("cursor".into()),
        expected_revision: Some(42),
        limit: 7,
    })
    .unwrap();
    let current = Request::ScanRecordsPage {
        path: "world/path".into(),
        cursor: Some("cursor".into()),
        expected_revision: Some(42),
        limit: 7,
    };
    assert_eq!(bytes, rmp_serde::to_vec_named(&current).unwrap());
    let decoded: Request = rmp_serde::from_slice(&bytes).unwrap();
    assert!(matches!(
        decoded,
        Request::ScanRecordsPage {
            expected_revision: Some(42),
            limit: 7,
            ..
        }
    ));
}
#[test]
fn absent_record_and_empty_collection_remain_distinct() {
    let absent = rmp_serde::to_vec_named(&Reply::Record(None)).unwrap();
    let empty = rmp_serde::to_vec_named(&Reply::Records(vec![])).unwrap();
    assert_ne!(absent, empty);
    assert!(matches!(
        rmp_serde::from_slice::<Reply>(&absent).unwrap(),
        Reply::Record(None)
    ));
    assert!(
        matches!(rmp_serde::from_slice::<Reply>(&empty).unwrap(),Reply::Records(rows) if rows.is_empty())
    );
}
