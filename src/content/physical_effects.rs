//! Read-only physical receipt query envelopes. Validation confers no authority.
use serde::{Deserialize, Serialize};
pub use univers_aip_contracts_data_effects::artifact_effects::{
    ArtifactEffectError, ArtifactEffectResult, ArtifactPhysicalEffectObservation,
    ArtifactPhysicalEffectQuery, ARTIFACT_PHYSICAL_EFFECT_CAPABILITY,
    MAX_ARTIFACT_EFFECT_REQUEST_BYTES,
};

pub const ENDPOINT: &str = "/v1/content/artifact-physical-effects/query";
pub const CAPABILITIES_ENDPOINT: &str = "/v1/content/artifact-physical-effects/capabilities";
pub const CLIENT_URL: &str = "http://content-engine/v1/content/artifact-physical-effects/query";
pub const SCHEMA: &str = "univers.content-artifact-physical-effects/v1";
pub const MAX_REQUEST_BYTES: usize = MAX_ARTIFACT_EFFECT_REQUEST_BYTES;
pub const MAX_RESPONSE_BYTES: usize = MAX_ARTIFACT_EFFECT_REQUEST_BYTES;
pub const MAX_ERROR_REASON_BYTES: usize = 1024;

/// Capability advertisement is a provider observation, never a permission. A
/// provider advertises Query only after genuine original receipt query support;
/// compiled features, old Content capability names and this DTO do not imply it.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Support {
    Unsupported,
    Query,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Capabilities {
    pub schema: String,
    pub capability: String,
    pub support: Support,
}
impl Capabilities {
    pub fn validate(&self) -> ArtifactEffectResult<()> {
        if self.schema != SCHEMA || self.capability != ARTIFACT_PHYSICAL_EFFECT_CAPABILITY {
            return Err(ArtifactEffectError::Unsupported);
        }
        Ok(())
    }
}

/// Request bytes are EXACT Data physical-query JSON. Its strict bounded decoder
/// rejects unknown fields/trailing bytes and retains the full original selector.
/// No Content write, repair, close, actor, token or issuer field is added here.
pub type Request = ArtifactPhysicalEffectQuery;
pub fn decode_request(bytes: &[u8]) -> ArtifactEffectResult<Request> {
    Request::from_json(bytes)
}
pub fn encode_request(request: &Request) -> ArtifactEffectResult<Vec<u8>> {
    request.to_json()
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    Invalid,
    Conflict,
    Unsupported,
    Denied,
    ResourceExhausted,
    SnapshotInvalidated,
    Unavailable,
}
/// Errors never assert durable rejection, no effects or an attempt seal.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Error {
    pub code: ErrorCode,
    pub reason: Option<String>,
}
impl Error {
    pub fn validate(&self) -> ArtifactEffectResult<()> {
        if let Some(reason) = &self.reason {
            if reason.is_empty()
                || reason.len() > MAX_ERROR_REASON_BYTES
                || reason.trim() != reason
                || reason.chars().any(char::is_control)
            {
                return Err(ArtifactEffectError::Invalid(
                    "invalid physical query error reason".into(),
                ));
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum Outcome {
    Observation {
        observation: Box<ArtifactPhysicalEffectObservation>,
    },
    Error {
        error: Error,
    },
}
/// Every result/error preserves the EXACT sent selector. A reply from a trusted
/// channel still requires provider Auth/current lineage truth, independently of
/// these shape checks. There is no aggregate coverage or complete/no-effects.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Reply {
    pub schema: String,
    pub query: Request,
    pub outcome: Outcome,
}
impl Reply {
    pub fn validate_for(&self, request: &Request) -> ArtifactEffectResult<()> {
        request.validate()?;
        if self.schema != SCHEMA {
            return Err(ArtifactEffectError::Unsupported);
        }
        if &self.query != request {
            return Err(ArtifactEffectError::Conflict(
                "physical reply changed original query".into(),
            ));
        }
        match &self.outcome {
            Outcome::Observation { observation } => observation.validate_for(request)?,
            Outcome::Error { error } => error.validate()?,
        }
        if serde_json::to_vec(self)
            .map_err(|error| ArtifactEffectError::Invalid(error.to_string()))?
            .len()
            > MAX_RESPONSE_BYTES
        {
            return Err(ArtifactEffectError::ResourceExhausted);
        }
        Ok(())
    }
    pub fn from_json(bytes: &[u8], request: &Request) -> ArtifactEffectResult<Self> {
        if bytes.len() > MAX_RESPONSE_BYTES {
            return Err(ArtifactEffectError::ResourceExhausted);
        }
        let reply: Self = serde_json::from_slice(bytes)
            .map_err(|error| ArtifactEffectError::Invalid(error.to_string()))?;
        reply.validate_for(request)?;
        Ok(reply)
    }
    pub fn to_json(&self, request: &Request) -> ArtifactEffectResult<Vec<u8>> {
        self.validate_for(request)?;
        serde_json::to_vec(self).map_err(|error| ArtifactEffectError::Invalid(error.to_string()))
    }
}
