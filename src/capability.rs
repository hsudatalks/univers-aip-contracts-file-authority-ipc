//! Selected-World FileCapability wire values. No storage or admission implementation.
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use univers_aip_contracts_data::file::{
    FileCapabilityError, FileListQuery, FileMetadata, FileOrganizationId, FilePage,
    FilePageRequest, FileRangeRead, FileWriteCommand, ListOptions, ListResponse, WriteFileRequest,
    WriteFileResponse, MAX_FILE_WRITE_BYTES,
};
pub const ENDPOINT: &str = "/v1/file/capability";
pub const CLIENT_URL: &str = "http://file-capability/v1/file/capability";
pub const IPC_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
pub const MAX_REQUEST_BYTES: usize = MAX_FILE_WRITE_BYTES + 64 * 1024;

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
    pub world_instance_id: String,
    pub request: Request,
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
