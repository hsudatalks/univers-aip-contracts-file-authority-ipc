// Frozen old univers-content-ipc wire definitions; original source SHA256 aa3af3746c2668b47d887f8ab13da50e8b398c7daf1ebde8c937f7ab9e1dc7eb
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
}
