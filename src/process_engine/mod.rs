//! Opt-in adapter for the independently distributed NuxtJP V8 view worker.

mod config;
mod contract;
mod digest;
mod engine;
mod process;

pub use config::{ProcessEngineConfig, ProcessView};
pub use contract::{WorkerLimits, WorkerPayload};
pub use digest::file as worker_sha256;
pub use engine::ProcessViewEngine;
