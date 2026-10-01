//! Mirrors only the worker's versioned stdio data contract, never its V8 implementation.

use serde::{Deserialize, Serialize};

pub const REQUEST_SCHEMA: &str = "nuxtjp://v8-view-worker/request/v1";
pub const RESPONSE_SCHEMA: &str = "nuxtjp://v8-view-worker/response/v1";
pub const ENGINE_SCHEMA: &str = "nuxtjp://local-runtime/process-engine/v1";
pub const SCRIPT_ID: &str = "nuxtjp-service-card-v1";
pub const SCRIPT_SHA256: &str = "bd954b068b08f679b2c90c788b907100a184097693bbf24ddd119937d4119b8e";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerLimits {
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub heap_mib: u16,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerPayload {
    pub title: String,
    pub summary: String,
    pub status_label: String,
    pub highlighted: bool,
}

#[derive(Serialize)]
pub(super) struct WorkerRequest<'a> {
    pub schema: &'static str,
    pub request_id: &'a str,
    pub script_id: &'static str,
    pub script_sha256: &'static str,
    pub limits: &'a WorkerLimits,
    pub payload: &'a WorkerPayload,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WorkerResponse {
    pub schema: String,
    pub request_id: String,
    pub status: String,
    pub view: WorkerRenderedView,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WorkerRenderedView {
    pub title: String,
    pub summary: String,
    pub status_label: String,
    pub highlighted: bool,
    pub html: String,
}

impl WorkerLimits {
    pub(super) fn validate(&self) -> bool {
        (1..=1_048_576).contains(&self.input_bytes)
            && (1..=262_144).contains(&self.output_bytes)
            && (8..=128).contains(&self.heap_mib)
            && (10..=10_000).contains(&self.timeout_ms)
    }
}

impl WorkerPayload {
    pub fn validate(&self) -> Result<(), crate::HostError> {
        if bounded(&self.title, 120)
            && bounded(&self.summary, 2_000)
            && bounded(&self.status_label, 80)
        {
            Ok(())
        } else {
            Err(crate::HostError::Configuration(
                "worker input payload exceeds its closed scalar bounds".into(),
            ))
        }
    }
}

fn bounded(value: &str, maximum: usize) -> bool {
    !value.is_empty() && value.len() <= maximum && !value.chars().any(char::is_control)
}
