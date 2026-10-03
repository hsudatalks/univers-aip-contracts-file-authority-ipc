//! Reverse Content -> World reservation/journal verification transport.
//! Every decoded value remains UNTRUSTED. This crate installs no authority,
//! authenticated channel, SQL reservation, material witness or journal executor.
pub use crate::content::retention::{Invocation, Original, Retained, WorldTerminal};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use univers_aip_contracts_data::content_types::{
    ContentContractError, ContentDigest, ContentResult,
};

pub const PROVIDER: &str = "world";
pub const CAPABILITY: &str = "world-content-original-verifier-v1";
pub const ENDPOINT: &str = "/internal/v1/world-content-retention/verify";
pub const CLIENT_URL: &str =
    "http://world-content-verifier/internal/v1/world-content-retention/verify";
pub const SCHEMA: &str = "univers.world-content-original-verifier/v1";
pub const MAX_MESSAGE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_WITNESS_BYTES: usize = 1024 * 1024;
pub const MAX_JOURNAL_BYTES: usize = 1024 * 1024;

fn need(ok: bool, message: &str) -> ContentResult<()> {
    if ok {
        Ok(())
    } else {
        Err(ContentContractError::Invalid(message.into()))
    }
}
fn convert<T>(result: Result<T, impl std::fmt::Debug>) -> ContentResult<T> {
    result.map_err(|error| ContentContractError::Invalid(format!("{error:?}")))
}
fn hash(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn reason(value: &str) -> ContentResult<()> {
    need(
        !value.is_empty()
            && value.len() <= 1024
            && value.trim() == value
            && !value.chars().any(char::is_control),
        "invalid verifier failure reason",
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    ContentToWorld,
}
pub const DIRECTION: Direction = Direction::ContentToWorld;

/// Closed verifier operations. NONE creates a World reservation or mutates World.
/// Reference removal is deliberately not an operation in this version.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "operation",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Operation {
    VerifyAcquisition(Box<Original>),
    /// Original management over authenticated owner channel; no resealed proof.
    VerifyOriginal(Box<Retained>),
    /// Fresh separately authenticated observer + unchanged retained Original.
    InspectOriginal {
        retained: Box<Retained>,
        observer: Invocation,
    },
    VerifyTerminal {
        retained: Box<Retained>,
        terminal: Box<WorldTerminal>,
    },
    /// Separate CURRENT readonly caller proof; unchanged saved mutation proof is
    /// historical evidence, never the credential for this verification.
    VerifyOriginalWithObserver {
        retained: Box<Retained>,
        observer: Invocation,
    },
    /// Verify an ACTUAL recorded terminal with a separate CURRENT readonly caller.
    /// Neither this operation nor its witness drains, releases or mutates anything.
    VerifyTerminalWithObserver {
        retained: Box<Retained>,
        terminal: Box<WorldTerminal>,
        observer: Invocation,
    },
}
impl Operation {
    pub fn validate(&self) -> ContentResult<()> {
        match self {
            Self::VerifyAcquisition(original) => original.validate(),
            Self::VerifyOriginal(retained) => retained.validate(),
            Self::InspectOriginal { retained, observer } => {
                retained.validate()?;
                observer.validate_schema()?;
                need(
                    observer.is_pure_read(),
                    "inspection requires fresh pureRead/readonly observer intent",
                )?;
                let payload = observer.payload()?;
                let request = &payload["request"];
                let original = &retained.original.admission.correlation;
                let fence = &original.selected_world_fence;
                need(
                    request["organizationId"].as_str() == Some(&fence.organization_id)
                        && request["worldInstanceId"].as_str() == Some(&fence.world_instance_id)
                        && request["expectedWorldRevision"].as_u64() == Some(fence.revision),
                    "observer differs from original organization/World/fence",
                )?;
                // Cryptography/actor/credential/current read policy MUST be verified by World.
                Ok(())
            }
            Self::VerifyTerminal { retained, terminal } => terminal.validate_for(retained),
            Self::VerifyOriginalWithObserver { retained, observer } => {
                validate_original_observer(retained, observer)
            }
            Self::VerifyTerminalWithObserver {
                retained,
                terminal,
                observer,
            } => {
                validate_original_observer(retained, observer)?;
                terminal.validate_for(retained)
            }
        }
    }
}

/// Structural readonly input binding only. The opaque fresh headers MUST be
/// opened by World with current Auth and matched to original caller/credential/
/// delegation/provider/action/callback scope. No decoded value grants authority.
fn validate_original_observer(retained: &Retained, observer: &Invocation) -> ContentResult<()> {
    retained.validate()?;
    observer.validate_schema()?;
    use crate::content::retention::{SpatialBindingOperation, SubjectSourceOperation};
    let fence = &retained.original.admission.correlation.selected_world_fence;
    match (observer, &retained.original.invocation) {
        (Invocation::Spatial(current), Invocation::Spatial(original)) => {
            match (&current.operation, &original.operation) {
                (
                    SpatialBindingOperation::GetOrResume(read),
                    SpatialBindingOperation::Apply(saved),
                ) if read.read_only == Some(true) => {
                    convert(read.validate_for(saved))?;
                    convert(read.validate_live_fence(fence))
                }
                _ => need(
                    false,
                    "original verification requires readonly spatial GetOrResume",
                ),
            }
        }
        (Invocation::SubjectSource(current), Invocation::SubjectSource(original)) => {
            match (&current.operation, &original.operation) {
                (
                    SubjectSourceOperation::SubjectSourceGetOrResume(read),
                    SubjectSourceOperation::SubjectSourceSelect(saved),
                ) if read.read_only => convert(read.validate_for(saved, fence)),
                _ => need(
                    false,
                    "original verification requires readonly source GetOrResume",
                ),
            }
        }
        _ => need(false, "observer differs from original invocation family"),
    }
}

/// Explicit direction/schema/capability plus complete canonical input. No channel
/// secret, caller-selected issuer/socket, allow flag or proof-renewal field exists.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Request {
    pub schema_version: String,
    pub provider: String,
    pub capability: String,
    pub direction: Direction,
    pub operation: Operation,
}
impl Request {
    pub fn new(operation: Operation) -> Self {
        Self {
            schema_version: SCHEMA.into(),
            provider: PROVIDER.into(),
            capability: CAPABILITY.into(),
            direction: DIRECTION,
            operation,
        }
    }
    pub fn validate(&self) -> ContentResult<()> {
        need(
            self.schema_version == SCHEMA
                && self.provider == PROVIDER
                && self.capability == CAPABILITY
                && self.direction == DIRECTION,
            "unsupported World verifier schema/provider/capability/direction",
        )?;
        self.operation.validate()
    }
    /// New-acquisition shape/expiry bounds only. Real signed proof, current Auth,
    /// material/flight/reservation/policies and queued commands remain World checks.
    pub fn validate_acquisition_at(&self, now_unix_ms: u64) -> ContentResult<()> {
        self.validate()?;
        match &self.operation {
            Operation::VerifyAcquisition(original) => original.validate_acquisition_at(now_unix_ms),
            _ => need(
                false,
                "original inspection/management is not fresh acquisition",
            ),
        }
    }
    /// Against ACTUAL loaded original World journal binding, never the request body.
    /// Does not authenticate that journal or admit/restore/drain/release any work.
    pub fn validate_recorded(
        &self,
        recorded: &Retained,
        terminal: Option<&WorldTerminal>,
    ) -> ContentResult<()> {
        self.validate()?;
        let retained = match &self.operation {
            Operation::VerifyOriginal(value)
            | Operation::InspectOriginal {
                retained: value, ..
            }
            | Operation::VerifyTerminal {
                retained: value, ..
            }
            | Operation::VerifyOriginalWithObserver {
                retained: value, ..
            }
            | Operation::VerifyTerminalWithObserver {
                retained: value, ..
            } => value,
            Operation::VerifyAcquisition(_) => {
                return need(
                    false,
                    "acquisition requires actual original World flight verification",
                )
            }
        };
        retained.validate_for(recorded)?;
        if let Operation::VerifyTerminal {
            terminal: value, ..
        }
        | Operation::VerifyTerminalWithObserver {
            terminal: value, ..
        } = &self.operation
        {
            let actual = terminal.ok_or_else(|| {
                ContentContractError::Conflict(
                    "actual original terminal journal unavailable; keep held".into(),
                )
            })?;
            actual.validate_for(recorded)?;
            need(
                value.as_ref() == actual,
                "terminal differs from ACTUAL full original ACK/journal",
            )?;
        }
        Ok(())
    }
    pub fn decode_named(bytes: &[u8]) -> ContentResult<Self> {
        let value: Self = named(bytes)?;
        value.validate()?;
        Ok(value)
    }
    pub fn decode_json(bytes: &[u8]) -> ContentResult<Self> {
        let value: Self = json(bytes)?;
        value.validate()?;
        Ok(value)
    }
}

/// OPAQUE FULL physical reservation/witness and original World journal bytes.
/// Independent bounds and digests preserve non-JSON data/max-u64/private encoding.
/// Shape-valid data is NOT a MaterialReservation implementation or permission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FullWitness {
    pub witness_bytes: Vec<u8>,
    pub witness_digest: ContentDigest,
    pub original_journal_bytes: Vec<u8>,
    pub original_journal_digest: ContentDigest,
}
impl FullWitness {
    pub fn validate(&self) -> ContentResult<()> {
        need(
            !self.witness_bytes.is_empty()
                && self.witness_bytes.len() <= MAX_WITNESS_BYTES
                && hash(&self.witness_bytes) == self.witness_digest.as_str(),
            "missing/oversized/tampered FULL reservation witness",
        )?;
        need(
            !self.original_journal_bytes.is_empty()
                && self.original_journal_bytes.len() <= MAX_JOURNAL_BYTES
                && hash(&self.original_journal_bytes) == self.original_journal_digest.as_str(),
            "missing/oversized/tampered FULL original World journal",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "status",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Status {
    /// Unverified received evidence, even after decode/correlation succeeds.
    UntrustedWitness(Box<FullWitness>),
    Denied {
        reason: String,
    },
    Unavailable {
        reason: String,
    },
    RecoveryRequired {
        reason: String,
    },
}
impl Status {
    fn validate(&self) -> ContentResult<()> {
        match self {
            Self::UntrustedWitness(value) => value.validate(),
            Self::Denied { reason: value }
            | Self::Unavailable { reason: value }
            | Self::RecoveryRequired { reason: value } => reason(value),
        }
    }
}

/// UNTRUSTED transport response. FULL request is mandatory correlation on every
/// status, including denial/recovery. There is no constructor for a trusted token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Reply {
    pub request: Request,
    pub status: Status,
}
impl Reply {
    pub fn validate(&self) -> ContentResult<()> {
        self.request.validate()?;
        self.status.validate()
    }
    /// Full operation/schema/original/retained/observer/terminal byte correlation.
    /// Authentic sender + current World record/witness truth remain owner authority.
    pub fn validate_for(&self, sent: &Request) -> ContentResult<()> {
        self.validate()?;
        sent.validate()?;
        need(
            &self.request == sent,
            "World verifier response differs from FULL original/current observer correlation",
        )
    }
    pub fn decode_named(bytes: &[u8]) -> ContentResult<Self> {
        let value: Self = named(bytes)?;
        value.validate()?;
        Ok(value)
    }
    pub fn decode_json(bytes: &[u8]) -> ContentResult<Self> {
        let value: Self = json(bytes)?;
        value.validate()?;
        Ok(value)
    }
}

fn named<T: DeserializeOwned + Serialize>(bytes: &[u8]) -> ContentResult<T> {
    need(
        bytes.len() <= MAX_MESSAGE_BYTES,
        "World verifier frame exceeds byte bound",
    )?;
    let mut decoder = rmp_serde::Deserializer::new(bytes);
    decoder.set_max_depth(64);
    // Typed decode FIRST: duplicate fields cannot be discarded by Value.
    let typed = convert(T::deserialize(&mut decoder))?;
    need(
        decoder.get_ref().is_empty(),
        "trailing World verifier frame bytes",
    )?;
    let input: serde_json::Value = convert(rmp_serde::from_slice(bytes))?;
    need(
        shape(&input, &convert(serde_json::to_value(&typed))?),
        "unknown/nested/positional World verifier fields",
    )?;
    Ok(typed)
}
fn json<T: DeserializeOwned + Serialize>(bytes: &[u8]) -> ContentResult<T> {
    need(
        bytes.len() <= MAX_MESSAGE_BYTES,
        "World verifier frame exceeds byte bound",
    )?;
    let typed: T = convert(serde_json::from_slice(bytes))?;
    let input: serde_json::Value = convert(serde_json::from_slice(bytes))?;
    need(
        shape(&input, &convert(serde_json::to_value(&typed))?),
        "unknown/nested World verifier fields",
    )?;
    Ok(typed)
}
fn shape(input: &serde_json::Value, expected: &serde_json::Value) -> bool {
    match (input, expected) {
        (serde_json::Value::Object(a), serde_json::Value::Object(b)) => a
            .iter()
            .all(|(key, value)| b.get(key).is_some_and(|other| shape(value, other))),
        (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
            a.len() == b.len() && a.iter().zip(b).all(|(value, other)| shape(value, other))
        }
        (serde_json::Value::Object(_) | serde_json::Value::Array(_), _)
        | (_, serde_json::Value::Object(_) | serde_json::Value::Array(_)) => false,
        _ => true,
    }
}
