//! Versioned complete Files-owned held snapshot projection, not raw KV access.
//! All decoded data stays UNTRUSTED. Only World sources actual same-store rows;
//! Files' issued pure decoder interprets them without creating live authority.
use crate::content_world_verifier::{self as verifier, FullWitness, Operation, Original};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use univers_aip_contracts_data::content_types::{
    ContentContractError, ContentDigest, ContentResult,
};
use univers_file_governed_policy::{Access, SelectedStorage};
pub use verifier::{Direction, CAPABILITY, CLIENT_URL, DIRECTION, ENDPOINT, PROVIDER};
pub const SCHEMA: &str = "univers.world-content-files-policy-projection/v1";
pub const MAX_SNAPSHOT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_REQUEST_BYTES: usize = verifier::MAX_MESSAGE_BYTES + 64 * 1024;
pub const MAX_REPLY_BYTES: usize = 64 * 1024 * 1024;
fn need(ok: bool, message: &str) -> ContentResult<()> {
    if ok {
        Ok(())
    } else {
        Err(ContentContractError::Invalid(message.into()))
    }
}
fn convert<T>(value: Result<T, impl std::fmt::Debug>) -> ContentResult<T> {
    value.map_err(|e| ContentContractError::Invalid(format!("{e:?}")))
}
/// Full existing original request, never caller-selected policy keys/rows/store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub schema_version: String,
    pub provider: String,
    pub capability: String,
    pub direction: Direction,
    pub verification: verifier::Request,
}
impl Request {
    pub fn new(verification: verifier::Request) -> Self {
        Self {
            schema_version: SCHEMA.into(),
            provider: PROVIDER.into(),
            capability: CAPABILITY.into(),
            direction: DIRECTION,
            verification,
        }
    }
    pub fn original(&self) -> ContentResult<&Original> {
        match &self.verification.operation {
            Operation::VerifyAcquisition(original) => Ok(original),
            Operation::VerifyOriginalWithObserver { retained, .. }
            | Operation::VerifyTerminalWithObserver { retained, .. } => Ok(&retained.original),
            _ => Err(ContentContractError::Invalid(
                "projection requires acquisition or mandatory fresh-observer Original/Terminal"
                    .into(),
            )),
        }
    }
    pub fn validate(&self) -> ContentResult<()> {
        need(
            self.schema_version == SCHEMA
                && self.provider == PROVIDER
                && self.capability == CAPABILITY
                && self.direction == DIRECTION,
            "unsupported policy projection schema/provider/capability/direction",
        )?;
        self.verification.validate()?;
        self.original()?;
        // Same complete nested request stays within legacy bounds in BOTH codecs.
        need(
            convert(rmp_serde::to_vec_named(&self.verification))?.len()
                <= verifier::MAX_MESSAGE_BYTES
                && convert(serde_json::to_vec(&self.verification))?.len()
                    <= verifier::MAX_MESSAGE_BYTES,
            "nested verification exceeds unchanged legacy request profile",
        )
    }
    pub fn validate_acquisition_at(&self, now_unix_ms: u64) -> ContentResult<()> {
        self.validate()?;
        self.verification.validate_acquisition_at(now_unix_ms)
    }
    pub fn decode_named(bytes: &[u8]) -> ContentResult<Self> {
        let value: Self = verifier::decode_named_bounded(bytes, MAX_REQUEST_BYTES)?;
        value.validate()?;
        Ok(value)
    }
    pub fn decode_json(bytes: &[u8]) -> ContentResult<Self> {
        let value: Self = verifier::decode_json_bounded(bytes, MAX_REQUEST_BYTES)?;
        value.validate()?;
        Ok(value)
    }
}
/// UNTRUSTED complete response projection. The store is an identity claim, never
/// a path/database to open. No Access/SelectedStorage trusted port is serialized.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FullPolicyProjection {
    pub original_operating_store_identity: String,
    pub files_held_policy_epoch: u64,
    pub files_held_policy_snapshot: Vec<u8>,
    pub files_held_policy_snapshot_digest: ContentDigest,
    pub witness: FullWitness,
}
impl FullPolicyProjection {
    pub fn validate(&self) -> ContentResult<()> {
        let store = &self.original_operating_store_identity;
        need(
            !store.is_empty()
                && store.len() <= 4096
                && store.trim() == store
                && !store.chars().any(char::is_control),
            "missing/invalid original operating store identity",
        )?;
        need(
            self.files_held_policy_epoch > 0
                && !self.files_held_policy_snapshot.is_empty()
                && self.files_held_policy_snapshot.len() <= MAX_SNAPSHOT_BYTES
                && self.files_held_policy_snapshot_digest.as_str()
                    == format!(
                        "sha256:{:x}",
                        Sha256::digest(&self.files_held_policy_snapshot)
                    ),
            "missing/oversized/tampered complete Files snapshot/epoch",
        )?;
        self.witness.validate()
    }
    /// PURE historical interpretation via Files' issued public decoder. Inputs
    /// MUST be actual trusted selection and unchanged owner-recorded Access, not
    /// reconstructed from wire. This returns epoch, never current permission.
    pub fn validate_policy_for(
        &self,
        selected: &SelectedStorage,
        original: &Access,
    ) -> ContentResult<u64> {
        self.validate()?;
        need(
            self.original_operating_store_identity == selected.storage_identity,
            "projection differs from actually selected original operating store",
        )?;
        let epoch = convert(
            univers_file_governed_policy::snapshot::validate_held_policy_snapshot(
                selected,
                original,
                &self.files_held_policy_snapshot,
            ),
        )?;
        need(
            epoch == self.files_held_policy_epoch,
            "projection epoch differs from complete Files-owned original snapshot",
        )?;
        Ok(epoch)
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "status",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Status {
    UntrustedProjection(Box<FullPolicyProjection>),
    Denied { reason: String },
    Unavailable { reason: String },
    RecoveryRequired { reason: String },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Reply {
    pub request: Request,
    pub status: Status,
}
impl Reply {
    pub fn validate(&self) -> ContentResult<()> {
        self.request.validate()?;
        match &self.status {
            Status::UntrustedProjection(value) => {
                value.validate()?;
                if let Operation::VerifyTerminalWithObserver { terminal, .. } =
                    &self.request.verification.operation
                {
                    need(
                        value.witness.original_journal_bytes == terminal.original_world_journal
                            && value.witness.original_journal_digest
                                == terminal.original_world_journal_digest,
                        "projection differs from complete original terminal journal",
                    )?;
                }
                Ok(())
            }
            Status::Denied { reason }
            | Status::Unavailable { reason }
            | Status::RecoveryRequired { reason } => verifier::reason(reason),
        }
    }
    pub fn validate_for(&self, sent: &Request) -> ContentResult<()> {
        self.validate()?;
        sent.validate()?;
        need(&self.request == sent, "projection reply differs from FULL original/retained/current observer/terminal request")
    }
    /// This correlates data only. Caller still authenticates actual producer,
    /// current Auth and all current same-store predicates before/after awaits.
    pub fn validate_policy_for(
        &self,
        sent: &Request,
        selected: &SelectedStorage,
        original: &Access,
    ) -> ContentResult<u64> {
        self.validate_for(sent)?;
        let canonical = sent.original()?;
        let correlation = &canonical.admission.correlation;
        need(
            original.organization == selected.organization
                && original.world == selected.world
                && original.selected_world_fence == correlation.selected_world_fence
                && original.expected_material == canonical.material
                && original.runtime_identity == canonical.world_runtime.runtime_instance_id
                && original.principal == correlation.security_context.principal_id
                && Some(original.credential.as_str())
                    == correlation.security_context.credential_id.as_deref(),
            "actual original Access differs from full canonical request",
        )?;
        match &self.status {
            Status::UntrustedProjection(value) => value.validate_policy_for(selected, original),
            _ => Err(ContentContractError::Conflict(
                "projection denied/unavailable/uncertain; keep held".into(),
            )),
        }
    }
    pub fn decode_named(bytes: &[u8]) -> ContentResult<Self> {
        let value: Self = verifier::decode_named_bounded(bytes, MAX_REPLY_BYTES)?;
        value.validate()?;
        Ok(value)
    }
    pub fn decode_json(bytes: &[u8]) -> ContentResult<Self> {
        let value: Self = verifier::decode_json_bounded(bytes, MAX_REPLY_BYTES)?;
        value.validate()?;
        Ok(value)
    }
}
