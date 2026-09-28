//! Pure public envelopes for the World-owned File authority protocol.
//! Request scope, storage, grants, recovery, retries and admission belong to owners.
#[cfg(feature = "capability")]
pub mod capability;
pub const ENDPOINT: &str = "/v1/file-authority";
pub const CLIENT_URL: &str = "http://file-authority/v1/file-authority";
pub const CAPABILITY: &str = "file-authority";
pub const IPC_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
pub const MAX_FILE_AUTHORITY_REQUEST_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_FILE_AUTHORITY_METADATA_RESPONSE_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_FILE_AUTHORITY_COLLECTION_RESPONSE_BYTES: usize = 16 * 1024 * 1024;
use serde::{Deserialize, Serialize};
use univers_aip_contracts_data::content_types::{ContentCapability, ContentDigest};
use univers_aip_contracts_data::file::{
    ContentGrantRequest, FileAuthorityError, FileCatalogBatch, FileCatalogPage, FileCatalogRecord,
    FileContentReferencePage, FileContentWriteClaim, FileContentWriteFinalizeStatus,
    FileContentWriteIntent, FileContentWriteMutation, FileDirectoryRename,
    FileDirectoryRenameAdvanceRequest, FileDirectoryRenameBeginRequest, FileDirectoryRenameClaim,
    FileDirectoryRenameManifestChunk, FileDirectoryRenameMetadataAckRequest,
    FileDirectoryRenamePage, FileDirectoryRenameRecoveryRequest, FileEmptyTrash,
    FileEmptyTrashAccountRequest, FileEmptyTrashBeginRequest, FileEmptyTrashChildPage,
    FileEmptyTrashClaim, FileEmptyTrashDispatchRequest, FileEmptyTrashManifestRequest,
    FileEmptyTrashPage, FileEmptyTrashRecoveryRequest, FileIdempotencyClaim,
    FileIdempotencyClaimRequest, FileIdempotencyReceipt, FileIdempotencyRecoveryReport,
    FileIdempotencyRecoveryRequest, FileMigrationReceipt, FileNamespaceTransition,
    FileNamespaceTransitionAdvanceRequest, FileNamespaceTransitionBeginRequest,
    FileNamespaceTransitionClaim, FileNamespaceTransitionKind, FileNamespaceTransitionPage,
    FileNamespaceTransitionRecoveryRequest, FilePurge, FilePurgeBeginRequest, FilePurgeClaim,
    FilePurgeCompleteRequest, FilePurgeGcClaim, FilePurgeGcRequest, FilePurgePage,
    FilePurgeRecoveryRequest, FilePurgeSealRequest, FileQuotaClaimPage, FileQuotaClaimReceipt,
    FileQuotaClaimRequest, FileQuotaReservationReceipt, FileQuotaReservationRequest,
    FileQuotaStatus, FileRetentionAuditReceipt, FileRetentionAuditRequest, FileRetentionDecision,
    FileRetentionDecisionRequest, FileRetentionPolicy,
};
use univers_aip_contracts_data::storage::file_store::{
    FileContentReconciliationMode, FileContentReconciliationReport,
};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    ReadProbe,
    GetRecord(String),
    ScanRecords(String),
    PutRecord(FileCatalogRecord),
    CreateRecord(FileCatalogRecord),
    DeleteRecord(String),
    ApplyBatch(FileCatalogBatch),
    BeginContentWrite(FileContentWriteIntent),
    PendingContentWrites(usize),
    ContentWriteForMutation(FileContentWriteMutation),
    FinalizeContentWrite(FileContentWriteIntent),
    AbandonContentWrite(FileContentWriteIntent),
    MigrationReceipt(String),
    CompleteMigration(FileMigrationReceipt),
    ClaimIdempotency(FileIdempotencyClaimRequest),
    CompleteIdempotency {
        owner_id: String,
        receipt: FileIdempotencyReceipt,
    },
    ReleaseIdempotency {
        key: String,
        request_digest: String,
        owner_id: String,
    },
    RecoverIdempotency(FileIdempotencyRecoveryRequest),
    QuotaStatus,
    SetQuotaLimit(u64),
    ReserveQuota(FileQuotaReservationRequest),
    ReleaseQuota(String),
    ClaimQuotaUsage(FileQuotaClaimRequest),
    ReleaseQuotaUsage(String),
    IssueContentGrant(ContentGrantRequest),
    ReferencedContent,
    ScanRecordsPage {
        path: String,
        cursor: Option<String>,
        expected_revision: Option<u64>,
        limit: usize,
    },
    ReferencedContentPage {
        cursor: Option<String>,
        limit: usize,
    },
    QuotaUsageClaimsPage {
        cursor: Option<String>,
        limit: usize,
    },
    BeginNamespaceTransition(FileNamespaceTransitionBeginRequest),
    AdvanceNamespaceTransition(FileNamespaceTransitionAdvanceRequest),
    ClaimNamespaceTransitionRecovery(FileNamespaceTransitionRecoveryRequest),
    NamespaceTransition(String),
    NamespaceTransitionForIdempotency {
        key: String,
        kind: FileNamespaceTransitionKind,
        request_digest: String,
    },
    PendingNamespaceTransitionsPage {
        cursor: Option<String>,
        limit: usize,
    },
    BeginDirectoryRename(FileDirectoryRenameBeginRequest),
    AdvanceDirectoryRename(FileDirectoryRenameAdvanceRequest),
    ClaimDirectoryRenameRecovery(FileDirectoryRenameRecoveryRequest),
    NextDirectoryRenameManifestChunk(String),
    AckDirectoryRenameMetadataChunk(FileDirectoryRenameMetadataAckRequest),
    DirectoryRename(String),
    DirectoryRenameForIdempotency {
        key: String,
        request_digest: String,
    },
    PendingDirectoryRenamesPage {
        cursor: Option<String>,
        limit: usize,
    },
    BeginPurge(FilePurgeBeginRequest),
    SealPurgeReferences(FilePurgeSealRequest),
    ClaimPurgeGc(FilePurgeGcRequest),
    CompletePurge(FilePurgeCompleteRequest),
    ClaimPurgeRecovery(FilePurgeRecoveryRequest),
    Purge(String),
    PurgeForIdempotency {
        key: String,
        request_digest: String,
    },
    PendingPurgesPage {
        cursor: Option<String>,
        limit: usize,
    },
    BeginEmptyTrash(FileEmptyTrashBeginRequest),
    ManifestNextEmptyTrashPage(FileEmptyTrashManifestRequest),
    DispatchNextEmptyTrashChildren(FileEmptyTrashDispatchRequest),
    AccountEmptyTrashChildren(FileEmptyTrashAccountRequest),
    ClaimEmptyTrashRecovery(FileEmptyTrashRecoveryRequest),
    EmptyTrash(String),
    EmptyTrashForIdempotency {
        key: String,
        request_digest: String,
    },
    PendingEmptyTrashPage {
        cursor: Option<String>,
        limit: usize,
    },
    EmptyTrashChildPurgesPage {
        empty_trash_id: String,
        cursor: Option<String>,
        limit: usize,
    },
    RetentionPolicy,
    DecideRetention(FileRetentionDecisionRequest),
    RecordRetentionAudit(FileRetentionAuditRequest),
    RetentionAudit(String),
    /// Ask World to prepare an authoritative write before transferring bytes.
    PrepareContentWrite(FileContentWritePrepareRequest),
    EnsureDirectory(FileCatalogPathRequest),
    RemoveEmptyDirectory(FileCatalogPathRequest),
    DeleteFile(FileCatalogPathRequest),
    MoveFileRecord(FileRecordMoveRequest),
    /// World checks Content and decides repair; the caller selects mode and bound only.
    ReconcilePendingContentWrites {
        mode: FileContentReconciliationMode,
        limit: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Reply {
    Unit,
    Record(Option<FileCatalogRecord>),
    Records(Vec<FileCatalogRecord>),
    ContentWriteClaim(FileContentWriteClaim),
    ContentWriteIntents(Vec<FileContentWriteIntent>),
    ContentWriteIntent(Option<FileContentWriteIntent>),
    ContentWriteFinalized(FileContentWriteFinalizeStatus),
    Bool(bool),
    Migration(Option<FileMigrationReceipt>),
    IdempotencyClaim(FileIdempotencyClaim),
    IdempotencyReceipt(FileIdempotencyReceipt),
    IdempotencyRecovery(FileIdempotencyRecoveryReport),
    QuotaStatus(FileQuotaStatus),
    QuotaReservation(FileQuotaReservationReceipt),
    QuotaClaim(FileQuotaClaimReceipt),
    ContentGrant(ContentCapability),
    Digests(Vec<ContentDigest>),
    Error(FileAuthorityError),
    CatalogPage(FileCatalogPage),
    ContentReferencePage(FileContentReferencePage),
    QuotaClaimPage(FileQuotaClaimPage),
    NamespaceTransitionClaim(FileNamespaceTransitionClaim),
    NamespaceTransition(Option<FileNamespaceTransition>),
    NamespaceTransitionValue(FileNamespaceTransition),
    NamespaceTransitionPage(FileNamespaceTransitionPage),
    DirectoryRenameClaim(FileDirectoryRenameClaim),
    DirectoryRename(Option<FileDirectoryRename>),
    DirectoryRenameValue(FileDirectoryRename),
    DirectoryRenamePage(FileDirectoryRenamePage),
    DirectoryRenameManifestChunk(Option<FileDirectoryRenameManifestChunk>),
    PurgeClaim(FilePurgeClaim),
    Purge(Option<FilePurge>),
    PurgeValue(FilePurge),
    PurgeGcClaim(FilePurgeGcClaim),
    PurgePage(FilePurgePage),
    EmptyTrashClaim(FileEmptyTrashClaim),
    EmptyTrash(Option<FileEmptyTrash>),
    EmptyTrashValue(FileEmptyTrash),
    EmptyTrashPage(FileEmptyTrashPage),
    EmptyTrashChildPage(FileEmptyTrashChildPage),
    RetentionPolicy(FileRetentionPolicy),
    RetentionDecision(FileRetentionDecision),
    RetentionAuditReceipt(FileRetentionAuditReceipt),
    RetentionAudit(Option<FileRetentionAuditReceipt>),
    ContentWritePrepared(FileContentWritePrepareOutcome),
    DirectoryEnsured {
        record: FileCatalogRecord,
        created: bool,
    },
    DirectoryRemoved {
        removed: bool,
    },
    FileDeleted {
        deleted: bool,
    },
    FileRecordMoved {
        record: Option<FileCatalogRecord>,
        changed: bool,
    },
    ContentWritesReconciled(FileContentReconciliationReport),
}

/// Content byte-transfer protocol, distinct from File-authority metadata.
pub mod content;

/// Caller-declared condition, evaluated against World's authoritative catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileContentWritePrepareMode {
    Replace,
    CreateOnly,
    IfHashMatches(ContentDigest),
}

/// Declares desired content, not catalog authority or a trusted mutation receipt.
/// The receiving World binds scope and validates the optional mutation link.
/// Intent identity, timestamps and previous catalog state are never caller inputs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileContentWritePrepareRequest {
    pub path: String,
    pub digest: ContentDigest,
    pub size_bytes: u64,
    pub mode: FileContentWritePrepareMode,
    pub mutation: Option<FileContentWriteMutation>,
}

/// A false condition does not claim a write or authorize any byte transfer.
/// Claimed reuses the canonical C0 Data claim, including original retry evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileContentWritePrepareOutcome {
    Claimed(Box<FileContentWriteClaim>),
    ConditionNotMet,
}

/// Path intent only. World binds scope and chooses records, time and CAS conditions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileCatalogPathRequest {
    pub path: String,
}

/// One file-record move for repair steps; not a namespace Move workflow.
/// The World response reports this call's result, never a durable retry receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FileRecordMoveRequest {
    pub source_path: String,
    pub target_path: String,
}
