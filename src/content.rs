//! Pure Content Engine wire envelopes. Owners verify capabilities and perform
//! persistence; decoding these values does not authorize an operation.
use serde::{Deserialize, Serialize};
use univers_aip_contracts_data::content_types::{
    ContentContractError, ContentDeleteReceipt, ContentDeleteRequest, ContentDescriptor,
    ContentHeadRequest, ContentPutReceipt, ContentPutRequest, ContentReadReceipt,
    ContentReadRequest, ContentUploadAbortReceipt, ContentUploadAbortRequest,
    ContentUploadBeginReceipt, ContentUploadBeginRequest, ContentUploadChunkReceipt,
    ContentUploadChunkRequest, ContentUploadCommitRequest, ContentVerifyReceipt,
    ContentVerifyRequest,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation", content = "value", rename_all = "snake_case")]
pub enum Request {
    ReadProbe,
    Put(Box<ContentPutRequest>),
    BeginUpload(ContentUploadBeginRequest),
    PutUploadChunk(Box<ContentUploadChunkRequest>),
    CommitUpload(ContentUploadCommitRequest),
    AbortUpload(ContentUploadAbortRequest),
    Head(ContentHeadRequest),
    Read(ContentReadRequest),
    Verify(ContentVerifyRequest),
    Delete(ContentDeleteRequest),
    /// Explicit negotiated retention lane; no legacy fallback.
    #[cfg(feature = "retention")]
    Retention(Box<retention::Request>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", content = "value", rename_all = "snake_case")]
pub enum Reply {
    Unit,
    Put(ContentPutReceipt),
    UploadBegin(ContentUploadBeginReceipt),
    UploadChunk(ContentUploadChunkReceipt),
    UploadAbort(ContentUploadAbortReceipt),
    Descriptor(ContentDescriptor),
    Read(ContentReadReceipt),
    Verify(ContentVerifyReceipt),
    Delete(ContentDeleteReceipt),
    Error(ContentContractError),
    #[cfg(feature = "retention")]
    Retention(Box<retention::Reply>),
}

pub const ENDPOINT: &str = "/v1/content";
pub const CLIENT_URL: &str = "http://content-engine/v1/content";
pub const CAPABILITY: &str = "content-engine";
pub const CONTENT_CHUNK_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_CONTENT_REQUEST_BYTES: usize = 10 * 1024 * 1024;

/// Optional canonical original Content retention wire.
#[cfg(feature = "retention")]
pub mod retention;
