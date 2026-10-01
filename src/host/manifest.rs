use super::{EngineManifest, LocalRuntimeHost};
use crate::{HostError, RuntimeCapability, RuntimeLimits, RuntimeMode};
use serde::Serialize;

#[derive(Serialize)]
struct SecurityManifest<'a> {
    default_band: &'static str,
    external_egress: &'static str,
    allowed_origins: &'a [String],
    limits: &'a RuntimeLimits,
}

#[derive(Serialize)]
struct Manifest<'a> {
    schema: &'static str,
    runtime_id: &'a str,
    protocol_version: &'static str,
    mode: RuntimeMode,
    external_actions: bool,
    engine: EngineManifest,
    security: SecurityManifest<'a>,
    capabilities: &'a [RuntimeCapability],
}

impl LocalRuntimeHost {
    pub fn manifest_json(&self) -> Result<Vec<u8>, HostError> {
        let manifest = Manifest {
            schema: "nuxtjp://local-runtime/manifest/v1",
            runtime_id: &self.config.runtime_id,
            protocol_version: "1",
            mode: self.config.mode,
            external_actions: false,
            engine: EngineManifest {
                kind: "rust-v8",
                status: self.engine.status(),
                isolation: if self.engine.status() == crate::EngineStatus::Ready {
                    "process"
                } else {
                    "in-process-simulation"
                },
                arbitrary_network: false,
                arbitrary_filesystem: false,
            },
            security: SecurityManifest {
                default_band: "sealed",
                external_egress: "deny",
                allowed_origins: &self.config.allowed_origins,
                limits: &self.config.limits,
            },
            capabilities: &self.config.capabilities,
        };
        self.bounded_json(&manifest)
    }
}
