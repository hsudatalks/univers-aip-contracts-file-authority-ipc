use serde_json::Value;
use univers_aip_contracts_file_authority_ipc::{self as ipc, content};
fn compare(input: &str, expected: usize) {
    let cases: Value = serde_json::from_str(input).unwrap();
    let rows = cases.as_array().unwrap();
    assert_eq!(rows.len(), expected);
    for row in rows {
        let bytes: Vec<u8> = serde_json::from_value(row["bytes"].clone()).unwrap();
        match row["kind"].as_str().unwrap() {
            "content_request" => {
                let current: content::Request =
                    serde_json::from_value(row["value"].clone()).unwrap();
                assert_eq!(rmp_serde::to_vec_named(&current).unwrap(), bytes);
                let decoded: content::Request = rmp_serde::from_slice(&bytes).unwrap();
                assert_eq!(serde_json::to_value(decoded).unwrap(), row["value"]);
            }
            "content_reply" => {
                let current: content::Reply = serde_json::from_value(row["value"].clone()).unwrap();
                assert_eq!(rmp_serde::to_vec_named(&current).unwrap(), bytes);
                let decoded: content::Reply = rmp_serde::from_slice(&bytes).unwrap();
                assert_eq!(serde_json::to_value(decoded).unwrap(), row["value"]);
            }
            "file_request" => {
                let current: ipc::Request = serde_json::from_value(row["value"].clone()).unwrap();
                assert_eq!(rmp_serde::to_vec_named(&current).unwrap(), bytes);
                let decoded: ipc::Request = rmp_serde::from_slice(&bytes).unwrap();
                assert_eq!(serde_json::to_value(decoded).unwrap(), row["value"]);
            }
            "file_reply" => {
                let current: ipc::Reply = serde_json::from_value(row["value"].clone()).unwrap();
                assert_eq!(rmp_serde::to_vec_named(&current).unwrap(), bytes);
                let decoded: ipc::Reply = rmp_serde::from_slice(&bytes).unwrap();
                assert_eq!(serde_json::to_value(decoded).unwrap(), row["value"]);
            }
            _ => panic!("unknown actual published golden kind"),
        }
    }
}
#[test]
fn actual_registry_olddev1_default_named_content_scalar_and_read_wire_are_unchanged() {
    compare(include_str!("fixtures/olddev1_default_wire.json"), 33);
}
#[cfg(feature = "retention")]
#[test]
fn actual_registry_olddev1_optin_named_retention_wire_is_unchanged() {
    compare(include_str!("fixtures/olddev1_retention_wire.json"), 37);
}
