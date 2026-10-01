//! Product-neutral local UI host with a closed Rust/V8 engine boundary.

mod config;
mod engine;
mod error;
mod host;
mod http;
mod model;
mod payload;
mod process_engine;
mod security;
mod session;

pub use config::{HostConfig, load_config};
pub use engine::{ContractSimulationEngine, ViewEngine};
pub use error::HostError;
pub use host::LocalRuntimeHost;
pub use http::run_server;
pub use model::*;
pub use payload::{PayloadField, PayloadScalarKind, PayloadSchema};
pub use process_engine::{
    ProcessEngineConfig, ProcessView, ProcessViewEngine, WorkerLimits, WorkerPayload, worker_sha256,
};
pub use security::{BoundaryTarget, InformationBand, SessionBudget, band_allows_target};
