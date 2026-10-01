use crate::{InformationBand, PayloadSchema, security::SessionBudget};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeMode {
    LocalSimulation,
    LocalProduction,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineStatus {
    Disabled,
    Simulated,
    Ready,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeLimits {
    pub max_request_bytes: u64,
    pub max_response_bytes: u64,
    pub max_messages_per_minute: u64,
    pub max_bytes_per_minute: u64,
    pub session_ttl_seconds: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeCapability {
    pub id: String,
    pub service_id: String,
    pub description: String,
    pub output_band: InformationBand,
    pub payload_schema: PayloadSchema,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewFixture {
    pub capability_id: String,
    pub view_id: String,
    pub band: InformationBand,
    pub payload_schema_id: String,
    pub payload: Value,
}

#[derive(Clone, Debug)]
pub struct RenderedView {
    pub band: InformationBand,
    pub payload_schema_id: String,
    pub payload: Value,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionRequest {
    pub schema: String,
    pub origin: String,
    pub audience: String,
    pub client_nonce: String,
    pub requested_capabilities: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SessionGrant {
    pub schema: &'static str,
    pub session_id: String,
    pub token: String,
    pub subject_id: String,
    pub audience: String,
    pub expires_at_unix_seconds: u64,
    pub granted_capabilities: Vec<String>,
    pub classification_ceiling: InformationBand,
    pub budget: SessionBudget,
    pub external_actions: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewRequest {
    pub schema: String,
    pub capability_id: String,
    pub view_id: String,
    pub request_nonce: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct ViewEnvelope {
    pub schema: &'static str,
    pub capability_id: String,
    pub view_id: String,
    pub band: InformationBand,
    pub payload_schema_id: String,
    pub generated_at_unix_seconds: u64,
    pub expires_at_unix_seconds: u64,
    pub external_actions: bool,
    pub payload: Value,
}
