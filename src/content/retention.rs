//! Canonical Content retention envelopes. Shape/correlation checks mint no
//! authentication, live ACL, physical enrollment, reservation or drainage proof.
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};
use univers_aip_contracts_data::content_types::{
    ContentContractError, ContentDescriptor, ContentDigest, ContentResult,
};
pub use univers_aip_contracts_world_ipc::{
    spatial_mutation::{
        SpatialBindingAssetProof, SpatialBindingCommitAdmission, SpatialBindingInvocation,
        SpatialBindingKernelIdentity, SpatialBindingOperation, SpatialBindingOutcome,
        SpatialBindingQuiescentOutcome,
    },
    spatial_reconciliation::{SpatialReconciliationAckFrame, SpatialReconciliationRequestFrame},
    subject_source::{SubjectSourceInvocation, SubjectSourceOperation, SubjectSourceOutcome},
};

pub const SCHEMA: &str = "univers.content-retention/v1";
pub const CAPABILITY: &str = "content-original-retention-v1";
pub const MAX_BODY_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_REQUEST_BYTES: usize = 8 * 1024 * 1024;
// Plain named byte arrays require at most two MessagePack bytes per body byte.
pub const MAX_RESPONSE_BYTES: usize = 2 * MAX_BODY_BYTES + MAX_REQUEST_BYTES;
pub const MAX_JOURNAL_BYTES: usize = 1024 * 1024;

fn require(ok: bool, message: &str) -> ContentResult<()> {
    if ok {
        Ok(())
    } else {
        Err(ContentContractError::Invalid(message.into()))
    }
}
fn canonical<T>(result: Result<T, impl std::fmt::Debug>) -> ContentResult<T> {
    result.map_err(|error| ContentContractError::Invalid(format!("{error:?}")))
}
fn identity(value: &str) -> ContentResult<()> {
    require(
        !value.is_empty()
            && value.len() <= 256
            && value.trim() == value
            && !value.chars().any(char::is_control),
        "invalid retention identity",
    )
}
fn proof(value: &str) -> ContentResult<()> {
    require(
        !value.is_empty() && value.len() <= MAX_JOURNAL_BYTES,
        "missing/oversized original proof",
    )
}
fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}

/// FULL published original input and signed header bytes; never a fresh DTO projection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Invocation {
    Spatial(Box<SpatialBindingInvocation>),
    SubjectSource(Box<SubjectSourceInvocation>),
}
impl Invocation {
    pub fn payload(&self) -> ContentResult<serde_json::Value> {
        match self {
            Self::Spatial(value) => canonical(value.operation.admission_payload()),
            Self::SubjectSource(value) => canonical(value.operation.admission_payload()),
        }
    }
    fn proofs(&self) -> (&str, &str) {
        match self {
            Self::Spatial(value) => (
                &value.forwarded_security_context,
                &value.runtime_action_admission,
            ),
            Self::SubjectSource(value) => (
                &value.forwarded_security_context,
                &value.runtime_action_admission,
            ),
        }
    }
    fn validate_schema(&self) -> ContentResult<()> {
        let valid = match self {
            Self::Spatial(value) => {
                value.schema_version == univers_aip_contracts_world_ipc::spatial_mutation::SCHEMA
            }
            Self::SubjectSource(value) => {
                value.schema_version == univers_aip_contracts_world_ipc::subject_source::SCHEMA
            }
        };
        require(valid, "unsupported original invocation schema")?;
        let (context, action) = self.proofs();
        proof(context)?;
        proof(action)?;
        self.payload()?;
        Ok(())
    }
    /// Intent only. Owners still verify original codec, fresh Auth and read grants.
    pub fn is_pure_read(&self) -> bool {
        match self {
            Self::Spatial(value) => {
                matches!(&value.operation, SpatialBindingOperation::GetOrResume(r) if r.read_only == Some(true))
            }
            Self::SubjectSource(value) => {
                matches!(
                    &value.operation,
                    SubjectSourceOperation::SubjectSourceRead(_)
                ) || matches!(&value.operation, SubjectSourceOperation::SubjectSourceGetOrResume(r) if r.read_only)
            }
        }
    }
}

/// Full canonical original admission, actual World runtime and immutable material.
/// Content scope/body MUST also match the actual World reservation and Files policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Original {
    pub invocation: Invocation,
    pub admission: SpatialBindingCommitAdmission,
    pub world_runtime: SpatialBindingKernelIdentity,
    pub material: SpatialBindingAssetProof,
    pub content: ContentDescriptor,
}
impl Original {
    pub fn validate(&self) -> ContentResult<()> {
        self.invocation.validate_schema()?;
        canonical(self.admission.validate())?;
        canonical(self.world_runtime.validate())?;
        self.content.scope.validate()?;
        let c = &self.admission.correlation;
        let payload = self.invocation.payload()?;
        canonical(c.validate_payload(&payload))?;
        require(
            c.provider_id == "world" && self.world_runtime.provider_id == c.provider_id,
            "original World provider/runtime mismatch",
        )?;
        require(
            c.security_context
                .credential_id
                .as_deref()
                .is_some_and(|id| !id.is_empty()),
            "original credential correlation required",
        )?;
        require(
            self.content.scope.organization_id == c.selected_world_fence.organization_id
                && self.content.scope.world_instance_id == c.selected_world_fence.world_instance_id,
            "Content differs from original organization/World",
        )?;
        let (context, action) = self.invocation.proofs();
        require(
            digest(context.as_bytes()) == c.forwarded_context_proof_digest
                && digest(action.as_bytes()) == c.action_proof_digest,
            "original signed proof bytes differ",
        )?;
        identity(&self.material.artifact_id)?;
        require(
            univers_aip_contracts_data::file::ArtifactRef::parse(
                self.material.artifact_ref.as_str(),
            )
            .is_some_and(|r| r == self.material.artifact_ref),
            "invalid immutable Artifact reference",
        )?;
        require(
            format!("artifacts:{}", self.material.artifact_ref.catalog_id())
                == self.material.artifact_id,
            "Artifact identity/reference mismatch",
        )?;
        identity(&self.material.content_type)?;
        let hash = ContentDigest::new(self.material.sha256.clone())?;
        require(
            self.material.size_bytes > 0
                && self.material.size_bytes <= MAX_BODY_BYTES as u64
                && self.content.size_bytes == self.material.size_bytes
                && self.content.digest == hash,
            "immutable material digest/size differs from Content",
        )?;
        match &self.invocation {
            Invocation::Spatial(value) => match &value.operation {
                SpatialBindingOperation::Apply(request) => require(
                    request.expected_asset == self.material,
                    "material differs from full original spatial input",
                ),
                _ => require(false, "only original spatial Apply may acquire Content"),
            },
            Invocation::SubjectSource(value) => match &value.operation {
                SubjectSourceOperation::SubjectSourceSelect(request) => require(
                    request.source.content_digest == self.material.sha256
                        && request.source.size_bytes == self.material.size_bytes,
                    "material differs from full original source input",
                ),
                _ => require(false, "only original source Select may acquire Content"),
            },
        }
    }
    /// Admission expiry only. This is NOT proof authentication or Files authorization.
    pub fn validate_acquisition_at(&self, now_unix_ms: u64) -> ContentResult<()> {
        self.validate()?;
        canonical(self.admission.validate_admission_at(now_unix_ms))
    }
}

/// Server-produced opaque ORIGINAL physical root + journal incarnation binding.
/// Trusted mounting resolves these IDs against concrete original files; no path or
/// caller-selected database is opened from this serializable value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PhysicalStore {
    pub storage_id: String,
    pub incarnation_id: String,
    pub journal_id: String,
    /// Digest over the owner's FULL actual original physical identity record.
    pub original_identity_digest: ContentDigest,
}
impl PhysicalStore {
    pub fn validate(&self) -> ContentResult<()> {
        identity(&self.storage_id)?;
        identity(&self.incarnation_id)?;
        identity(&self.journal_id)
    }
}

/// Original-only correlation, not a transferable lease or release authority.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Retained {
    pub original: Original,
    pub physical: PhysicalStore,
    pub generation: u64,
    /// Digest of the FULL recorded durable original pin/policy binding.
    pub original_record_digest: ContentDigest,
}
impl Retained {
    pub fn validate(&self) -> ContentResult<()> {
        self.original.validate()?;
        self.physical.validate()?;
        require(self.generation > 0, "zero original retention generation")
    }
    /// Full stored original equality, never rebase from ID/expiry/current state.
    pub fn validate_for(&self, recorded: &Self) -> ContentResult<()> {
        self.validate()?;
        recorded.validate()?;
        require(
            self == recorded,
            "retention differs from original physical generation/input/pin",
        )
    }
    fn validate_ack(&self, frame: &SpatialReconciliationAckFrame) -> ContentResult<()> {
        self.validate()?;
        require(
            frame.schema_version == univers_aip_contracts_world_ipc::spatial_reconciliation::SCHEMA,
            "unsupported canonical terminal ACK schema",
        )?;
        canonical(
            frame
                .acknowledgement
                .validate_for(&self.original.admission, &self.original.world_runtime),
        )?;
        match &frame.acknowledgement.outcome {
            SpatialBindingQuiescentOutcome::Terminal { receipt, .. } => {
                let Invocation::Spatial(value) = &self.original.invocation else {
                    return require(false, "spatial terminal ACK differs from original action");
                };
                let SpatialBindingOperation::Apply(request) = &value.operation else {
                    return require(false, "spatial terminal ACK requires original apply");
                };
                require(
                    receipt.expected_asset == self.original.material
                        && receipt.schema_version == request.schema_version
                        && receipt.subject_id == request.subject_id
                        && receipt.fields == request.fields
                        && receipt.source == request.source
                        && receipt.prior == request.prior,
                    "terminal result differs from complete original material/action/input",
                )
            }
            SpatialBindingQuiescentOutcome::SubjectSourceTerminal { receipt, .. } => {
                let Invocation::SubjectSource(value) = &self.original.invocation else {
                    return require(false, "source terminal ACK differs from original action");
                };
                let SubjectSourceOperation::SubjectSourceSelect(request) = &value.operation else {
                    return require(false, "source terminal ACK requires original select");
                };
                require(
                    receipt.request == **request,
                    "source terminal result differs from complete original input",
                )
            }
            _ => require(
                false,
                "terminal result required; staged/not-reserved cannot release retained body",
            ),
        }
    }
}
/// Complete original terminal journal and unchanged published ACK, not a digest-only
/// projection. The authenticated World owner validates actual durable journal truth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorldTerminal {
    pub acknowledgement: SpatialReconciliationAckFrame,
    pub original_world_journal: Vec<u8>,
    pub original_world_journal_digest: ContentDigest,
}
impl WorldTerminal {
    pub fn validate_for(&self, retained: &Retained) -> ContentResult<()> {
        retained.validate_ack(&self.acknowledgement)?;
        require(
            !self.original_world_journal.is_empty()
                && self.original_world_journal.len() <= MAX_JOURNAL_BYTES
                && digest(&self.original_world_journal)
                    == self.original_world_journal_digest.as_str(),
            "missing/oversized/tampered full original terminal journal",
        )
    }
}
fn accepted(terminal: &WorldTerminal) -> bool {
    let frame = &terminal.acknowledgement;
    matches!(&frame.acknowledgement.outcome, SpatialBindingQuiescentOutcome::Terminal { receipt, .. }
        if matches!(receipt.outcome, SpatialBindingOutcome::Accepted { .. }))
        || matches!(&frame.acknowledgement.outcome, SpatialBindingQuiescentOutcome::SubjectSourceTerminal { receipt, .. }
        if matches!(receipt.outcome, SubjectSourceOutcome::Selected { .. }))
}

/// Files-owned physical reference generation joined to the FULL accepted World ACK.
/// World chooses a genuine reference from its accepted result; Files verifies it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AcceptedReference {
    pub reference_id: String,
    pub generation: u64,
    pub accepted: WorldTerminal,
}
impl AcceptedReference {
    pub fn validate_for(&self, retained: &Retained) -> ContentResult<()> {
        identity(&self.reference_id)?;
        require(self.generation > 0, "zero accepted-reference generation")?;
        self.accepted.validate_for(retained)?;
        require(
            accepted(&self.accepted),
            "reference requires accepted full terminal result",
        )?;
        let accepted_id = match &self.accepted.acknowledgement.acknowledgement.outcome {
            SpatialBindingQuiescentOutcome::Terminal { receipt, .. } => match &receipt.outcome {
                SpatialBindingOutcome::Accepted { bundle_id, .. } => bundle_id.as_str(),
                _ => return require(false, "missing accepted spatial reference"),
            },
            SpatialBindingQuiescentOutcome::SubjectSourceTerminal { receipt, .. } => {
                match &receipt.outcome {
                    SubjectSourceOutcome::Selected { selection } => selection.selection_id.as_str(),
                    _ => return require(false, "missing accepted source reference"),
                }
            }
            _ => return require(false, "missing accepted terminal reference"),
        };
        require(
            self.reference_id == accepted_id,
            "reference identity differs from accepted result",
        )
    }
}

/// Content-owned removal evidence over EXACT recorded accepted reference.
/// Journal bytes are the original World-owned durable removal/checkpoint evidence,
/// not a replacement semantic DTO or a client declaration. Trusted World channel
/// must authenticate these exact bytes and prove removal + no future reattachment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceRemoval {
    pub reference: AcceptedReference,
    pub removal_id: String,
    pub world_runtime: SpatialBindingKernelIdentity,
    pub original_world_journal: Vec<u8>,
    pub original_world_journal_digest: ContentDigest,
}
impl ReferenceRemoval {
    pub fn validate_for(
        &self,
        retained: &Retained,
        recorded: &AcceptedReference,
    ) -> ContentResult<()> {
        self.reference.validate_for(retained)?;
        require(
            &self.reference == recorded,
            "reference removal differs from recorded reference generation/ACK",
        )?;
        identity(&self.removal_id)?;
        canonical(self.world_runtime.validate())?;
        require(
            self.world_runtime.provider_id == retained.original.world_runtime.provider_id,
            "reference removal owner differs",
        )?;
        require(
            !self.original_world_journal.is_empty()
                && self.original_world_journal.len() <= MAX_JOURNAL_BYTES
                && digest(&self.original_world_journal)
                    == self.original_world_journal_digest.as_str(),
            "missing/oversized/tampered full original reference removal journal",
        )
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "operation",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Request {
    Probe,
    Acquire(Box<Original>),
    /// Loads exact recorded original and rereads complete bytes; never reacquires.
    Restore(Box<Retained>),
    /// PureRead observer proof; no acquire, staging, cleanup or ledger mutation.
    Status {
        retained: Box<Retained>,
        observer: Invocation,
    },
    /// Authentic owner-channel housekeeping of recorded flight, including expired original.
    Drain {
        retained: Box<Retained>,
        request: SpatialReconciliationRequestFrame,
    },
    Terminal {
        retained: Box<Retained>,
        acknowledgement: WorldTerminal,
    },
    RetainReference {
        retained: Box<Retained>,
        reference: AcceptedReference,
    },
    ReleaseReference {
        retained: Box<Retained>,
        removal: ReferenceRemoval,
    },
}
impl Request {
    /// Structural checks only. EVERY owner action still requires real authentication.
    pub fn validate(&self) -> ContentResult<()> {
        match self {
            Self::Probe => Ok(()),
            Self::Acquire(original) => original.validate(),
            Self::Restore(retained) => retained.validate(),
            Self::Status { retained, observer } => {
                retained.validate()?;
                observer.validate_schema()?;
                require(
                    observer.is_pure_read(),
                    "status requires pureRead/readonly original observer",
                )?;
                let payload = observer.payload()?;
                let r = &payload["request"];
                let fence = &retained.original.admission.correlation.selected_world_fence;
                require(
                    r["organizationId"].as_str() == Some(&fence.organization_id)
                        && r["worldInstanceId"].as_str() == Some(&fence.world_instance_id)
                        && r["expectedWorldRevision"].as_u64() == Some(fence.revision),
                    "status observer differs from original scope/fence",
                )
            }
            Self::Drain { retained, request } => {
                retained.validate()?;
                require(
                    request.schema_version
                        == univers_aip_contracts_world_ipc::spatial_reconciliation::SCHEMA,
                    "unsupported canonical original drain schema",
                )?;
                canonical(request.request.validate_for(
                    &retained.original.admission,
                    &retained.original.world_runtime,
                ))
            }
            Self::Terminal {
                retained,
                acknowledgement,
            } => {
                acknowledgement.validate_for(retained)?;
                require(
                    !accepted(acknowledgement),
                    "accepted outcome must retain reference, never terminal-release",
                )
            }
            Self::RetainReference {
                retained,
                reference,
            } => reference.validate_for(retained),
            Self::ReleaseReference { retained, removal } => {
                removal.validate_for(retained, &removal.reference)
            }
        }
    }
    /// Owner dispatch checks against the actual original durable record, not body input.
    /// Acquisition is handled separately and can never use this original-management path.
    pub fn validate_recorded(
        &self,
        recorded: &Retained,
        reference: Option<&AcceptedReference>,
    ) -> ContentResult<()> {
        self.validate()?;
        let retained = match self {
            Self::Restore(retained)
            | Self::Status { retained, .. }
            | Self::Drain { retained, .. }
            | Self::Terminal { retained, .. }
            | Self::RetainReference { retained, .. }
            | Self::ReleaseReference { retained, .. } => retained,
            Self::Acquire(_) | Self::Probe => {
                return require(false, "acquire/probe is not original management")
            }
        };
        retained.validate_for(recorded)?;
        if let Self::ReleaseReference { removal, .. } = self {
            let reference = reference.ok_or_else(|| {
                ContentContractError::Conflict("original accepted reference unavailable".into())
            })?;
            removal.validate_for(recorded, reference)?;
        }
        if matches!(self, Self::Terminal { .. }) {
            require(reference.is_none(), "accepted reference remains held")?;
        }
        Ok(())
    }
    /// Strict complete Content envelope on the existing negotiated endpoint.
    pub fn decode_content_named(bytes: &[u8]) -> ContentResult<Self> {
        let outer: super::Request = strict_named(bytes, MAX_REQUEST_BYTES)?;
        let super::Request::Retention(value) = outer else {
            return Err(ContentContractError::Invalid(
                "expected retention Content lane".into(),
            ));
        };
        value.validate()?;
        Ok(*value)
    }
    pub fn decode_content_json(bytes: &[u8]) -> ContentResult<Self> {
        let outer: super::Request = strict_json(bytes, MAX_REQUEST_BYTES)?;
        let super::Request::Retention(value) = outer else {
            return Err(ContentContractError::Invalid(
                "expected retention Content lane".into(),
            ));
        };
        value.validate()?;
        Ok(*value)
    }
    pub fn decode_named(bytes: &[u8]) -> ContentResult<Self> {
        let value: Self = strict_named(bytes, MAX_REQUEST_BYTES)?;
        value.validate()?;
        Ok(value)
    }
    pub fn decode_json(bytes: &[u8]) -> ContentResult<Self> {
        let value: Self = strict_json(bytes, MAX_REQUEST_BYTES)?;
        value.validate()?;
        Ok(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Protocol {
    pub schema_version: String,
    pub capabilities: Vec<String>,
}
impl Protocol {
    /// Consumers require positive support; no fallback to head/delete/TTL.
    pub fn require_supported(&self) -> ContentResult<()> {
        require(
            self.schema_version == SCHEMA && self.capabilities.iter().any(|c| c == CAPABILITY),
            "original Content retention is unavailable",
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "state",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum State {
    Held,
    Referenced(Box<AcceptedReference>),
    Terminal(Box<WorldTerminal>),
    ReferenceReleased(Box<ReferenceRemoval>),
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "status",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Reply {
    Protocol(Protocol),
    /// FULL body; consumers also validate digest/size before use. No URL/head substitute.
    Body {
        retained: Box<Retained>,
        bytes: Vec<u8>,
    },
    Status {
        retained: Box<Retained>,
        state: Box<State>,
    },
    /// An authenticated owner drain request was received; body stays held.
    DrainPending(Box<Retained>),
    Terminal {
        retained: Box<Retained>,
        acknowledgement: WorldTerminal,
    },
    Referenced {
        retained: Box<Retained>,
        reference: AcceptedReference,
    },
    ReferenceReleased {
        retained: Box<Retained>,
        removal: ReferenceRemoval,
    },
    Error(ContentContractError),
}
impl Reply {
    pub fn validate(&self) -> ContentResult<()> {
        match self {
            Self::Protocol(value) => value.require_supported(),
            Self::Body { retained, bytes } => {
                retained.validate()?;
                require(
                    bytes.len() == retained.original.content.size_bytes as usize
                        && digest(bytes) == retained.original.content.digest.as_str(),
                    "incomplete/tampered original body",
                )
            }
            Self::Status { retained, state } => {
                retained.validate()?;
                match state.as_ref() {
                    State::Held => Ok(()),
                    State::Referenced(reference) => reference.validate_for(retained),
                    State::Terminal(acknowledgement) => Request::Terminal {
                        retained: retained.clone(),
                        acknowledgement: acknowledgement.as_ref().clone(),
                    }
                    .validate(),
                    State::ReferenceReleased(removal) => {
                        removal.validate_for(retained, &removal.reference)
                    }
                }
            }
            Self::DrainPending(retained) => retained.validate(),
            Self::Terminal {
                retained,
                acknowledgement,
            } => Request::Terminal {
                retained: retained.clone(),
                acknowledgement: acknowledgement.clone(),
            }
            .validate(),
            Self::Referenced {
                retained,
                reference,
            } => reference.validate_for(retained),
            Self::ReferenceReleased { retained, removal } => {
                removal.validate_for(retained, &removal.reference)
            }
            Self::Error(_) => Ok(()),
        }
    }
    /// Correlates response to the sent original. Sender/channel authentication is separate.
    pub fn validate_for(&self, request: &Request) -> ContentResult<()> {
        request.validate()?;
        self.validate()?;
        if matches!(self, Self::Error(_)) {
            return Ok(());
        }
        match (self, request) {
            (Self::Protocol(_), Request::Probe) => Ok(()),
            (Self::Body { retained, .. }, Request::Acquire(original)) => require(
                &retained.original == original.as_ref(),
                "acquire reply changed full original",
            ),
            (Self::Body { retained, .. }, Request::Restore(recorded))
            | (
                Self::Status { retained, .. },
                Request::Status {
                    retained: recorded, ..
                },
            )
            | (
                Self::DrainPending(retained),
                Request::Drain {
                    retained: recorded, ..
                },
            ) => retained.validate_for(recorded),
            (
                Self::Terminal {
                    retained,
                    acknowledgement,
                },
                Request::Terminal {
                    retained: recorded,
                    acknowledgement: expected,
                },
            ) => {
                retained.validate_for(recorded)?;
                require(
                    acknowledgement == expected,
                    "terminal reply changed full original ACK",
                )
            }
            (
                Self::Referenced {
                    retained,
                    reference,
                },
                Request::RetainReference {
                    retained: recorded,
                    reference: expected,
                },
            ) => {
                retained.validate_for(recorded)?;
                require(
                    reference == expected,
                    "reply changed accepted-reference generation/full result",
                )
            }
            (
                Self::ReferenceReleased { retained, removal },
                Request::ReleaseReference {
                    retained: recorded,
                    removal: expected,
                },
            ) => {
                retained.validate_for(recorded)?;
                require(
                    removal == expected,
                    "reply changed original reference removal",
                )
            }
            _ => require(false, "retention reply differs from request operation"),
        }
    }
    pub fn decode_named(bytes: &[u8]) -> ContentResult<Self> {
        require(
            bytes.len() <= MAX_RESPONSE_BYTES,
            "retention frame exceeds byte bound",
        )?;
        let mut decoder = rmp_serde::Deserializer::new(bytes);
        decoder.set_max_depth(64);
        let typed = canonical(Self::deserialize(&mut decoder))?;
        require(
            decoder.get_ref().is_empty(),
            "trailing retention frame bytes",
        )?;
        typed.validate()?;
        let mut shape_decoder = rmp_serde::Deserializer::new(bytes);
        shape_decoder.set_max_depth(64);
        let input = canonical(MetadataValue::deserialize(&mut shape_decoder))?.0;
        require(
            same_shape(&input, &typed.metadata()?),
            "positional or unknown nested retention fields",
        )?;
        Ok(typed)
    }
    pub fn decode_json(bytes: &[u8]) -> ContentResult<Self> {
        require(
            bytes.len() <= MAX_RESPONSE_BYTES,
            "retention frame exceeds byte bound",
        )?;
        let typed: Self = canonical(serde_json::from_slice(bytes))?;
        typed.validate()?;
        let input: MetadataValue = canonical(serde_json::from_slice(bytes))?;
        require(
            same_shape(&input.0, &typed.metadata()?),
            "unknown nested retention fields",
        )?;
        Ok(typed)
    }
    /// Strict complete Content reply; owner/channel authentication remains required.
    pub fn decode_content_named(bytes: &[u8]) -> ContentResult<Self> {
        require(
            bytes.len() <= MAX_RESPONSE_BYTES,
            "retention frame exceeds byte bound",
        )?;
        let mut decoder = rmp_serde::Deserializer::new(bytes);
        decoder.set_max_depth(64);
        let outer = canonical(super::Reply::deserialize(&mut decoder))?;
        require(
            decoder.get_ref().is_empty(),
            "trailing retention Content frame bytes",
        )?;
        let super::Reply::Retention(value) = outer else {
            return Err(ContentContractError::Invalid(
                "expected retention Content reply lane".into(),
            ));
        };
        value.validate()?;
        let mut shape_decoder = rmp_serde::Deserializer::new(bytes);
        shape_decoder.set_max_depth(64);
        let input = canonical(MetadataValue::deserialize(&mut shape_decoder))?.0;
        let expected = serde_json::json!({"status":"retention", "value":value.metadata()?});
        require(
            same_shape(&input, &expected),
            "positional or unknown nested Content retention fields",
        )?;
        Ok(*value)
    }
    pub fn decode_content_json(bytes: &[u8]) -> ContentResult<Self> {
        require(
            bytes.len() <= MAX_RESPONSE_BYTES,
            "retention frame exceeds byte bound",
        )?;
        let outer: super::Reply = canonical(serde_json::from_slice(bytes))?;
        let super::Reply::Retention(value) = outer else {
            return Err(ContentContractError::Invalid(
                "expected retention Content reply lane".into(),
            ));
        };
        value.validate()?;
        let input: MetadataValue = canonical(serde_json::from_slice(bytes))?;
        let expected = serde_json::json!({"status":"retention", "value":value.metadata()?});
        require(
            same_shape(&input.0, &expected),
            "unknown nested Content retention fields",
        )?;
        Ok(*value)
    }
    fn metadata(&self) -> ContentResult<serde_json::Value> {
        match self {
            Self::Body { retained, .. } => canonical(serde_json::to_value(Self::Body {
                retained: retained.clone(),
                bytes: Vec::new(),
            })),
            _ => canonical(serde_json::to_value(self)),
        }
    }
}

// Typed decode FIRST catches duplicate struct fields before a Value map can discard them.
fn strict_named<T: DeserializeOwned + Serialize>(bytes: &[u8], maximum: usize) -> ContentResult<T> {
    require(bytes.len() <= maximum, "retention frame exceeds byte bound")?;
    let mut decoder = rmp_serde::Deserializer::new(bytes);
    decoder.set_max_depth(64);
    let typed = canonical(T::deserialize(&mut decoder))?;
    require(
        decoder.get_ref().is_empty(),
        "trailing retention frame bytes",
    )?;
    let input: serde_json::Value = canonical(rmp_serde::from_slice(bytes))?;
    require(
        same_shape(&input, &canonical(serde_json::to_value(&typed))?),
        "positional or unknown nested retention fields",
    )?;
    Ok(typed)
}
fn strict_json<T: DeserializeOwned + Serialize>(bytes: &[u8], maximum: usize) -> ContentResult<T> {
    require(bytes.len() <= maximum, "retention frame exceeds byte bound")?;
    let typed: T = canonical(serde_json::from_slice(bytes))?;
    let input: serde_json::Value = canonical(serde_json::from_slice(bytes))?;
    require(
        same_shape(&input, &canonical(serde_json::to_value(&typed))?),
        "unknown nested retention fields",
    )?;
    Ok(typed)
}
fn same_shape(input: &serde_json::Value, normalized: &serde_json::Value) -> bool {
    match (input, normalized) {
        (serde_json::Value::Object(a), serde_json::Value::Object(b)) => a
            .iter()
            .all(|(key, value)| b.get(key).is_some_and(|other| same_shape(value, other))),
        (serde_json::Value::Array(a), serde_json::Value::Array(b)) => {
            a.len() == b.len()
                && a.iter()
                    .zip(b)
                    .all(|(value, other)| same_shape(value, other))
        }
        (serde_json::Value::Object(_) | serde_json::Value::Array(_), _)
        | (_, serde_json::Value::Object(_) | serde_json::Value::Array(_)) => false,
        _ => true,
    }
}

// Skip only complete-body bytes in the second structural pass. Typed decoding and
// SHA/size validation already check them; keep the key so unknown fields still fail.
// This avoids turning a 64 MiB body into millions of allocated JSON Value nodes.
struct MetadataValue(serde_json::Value);
impl<'de> Deserialize<'de> for MetadataValue {
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = MetadataValue;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("named retention metadata")
            }
            fn visit_map<M: serde::de::MapAccess<'de>>(
                self,
                mut map: M,
            ) -> Result<Self::Value, M::Error> {
                let mut value = serde_json::Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    let item = if key == "bytes" {
                        map.next_value::<serde::de::IgnoredAny>()?;
                        serde_json::Value::Array(Vec::new())
                    } else {
                        map.next_value::<MetadataValue>()?.0
                    };
                    value.insert(key, item);
                }
                Ok(MetadataValue(serde_json::Value::Object(value)))
            }
            fn visit_seq<S: serde::de::SeqAccess<'de>>(
                self,
                mut seq: S,
            ) -> Result<Self::Value, S::Error> {
                let mut value = Vec::new();
                while let Some(item) = seq.next_element::<MetadataValue>()? {
                    value.push(item.0);
                }
                Ok(MetadataValue(serde_json::Value::Array(value)))
            }
            fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(MetadataValue(value.into()))
            }
            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(MetadataValue(value.into()))
            }
            fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(MetadataValue(value.into()))
            }
            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(MetadataValue(value.into()))
            }
            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(MetadataValue(value.into()))
            }
            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Self::Value, E> {
                Ok(MetadataValue(serde_json::Value::from(value)))
            }
            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(MetadataValue(serde_json::Value::Null))
            }
            fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                self.visit_unit()
            }
        }
        decoder.deserialize_any(Visitor)
    }
}
