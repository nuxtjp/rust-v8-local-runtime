//! Loads and validates simulation or explicitly selected process-engine settings.

mod contracts;
mod load;
mod validate;

pub use load::load_config;

use crate::{
    EngineStatus, ProcessEngineConfig, ProcessView, RuntimeCapability, RuntimeLimits, RuntimeMode,
    ViewFixture,
};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostConfig {
    pub runtime_id: String,
    pub subject_id: String,
    pub bind: String,
    pub mode: RuntimeMode,
    pub external_actions: bool,
    pub external_egress: String,
    pub engine_status: EngineStatus,
    pub allowed_origins: Vec<String>,
    pub limits: RuntimeLimits,
    pub capabilities: Vec<RuntimeCapability>,
    #[serde(default)]
    pub views: Vec<ViewFixture>,
    #[serde(default)]
    pub process_engine: Option<ProcessEngineConfig>,
    #[serde(default)]
    pub process_views: Vec<ProcessView>,
}
