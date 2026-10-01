mod manifest;
mod session;
mod view;

use crate::{
    EngineStatus, HostConfig, HostError, ViewEngine, security::normalized_origin,
    session::SessionStore,
};
use serde::Serialize;
use std::sync::{Arc, Mutex};

pub struct LocalRuntimeHost {
    pub(super) config: HostConfig,
    pub(super) engine: Arc<dyn ViewEngine>,
    pub(super) sessions: Mutex<SessionStore>,
}

impl LocalRuntimeHost {
    pub fn new(config: HostConfig, engine: Arc<dyn ViewEngine>) -> Result<Self, HostError> {
        config.validate()?;
        if engine.status() != config.engine_status {
            return Err(HostError::Configuration(
                "engine status does not match configuration".to_string(),
            ));
        }
        Ok(Self {
            config,
            engine,
            sessions: Mutex::new(SessionStore::new()),
        })
    }

    pub fn bind(&self) -> &str {
        &self.config.bind
    }

    pub fn maximum_request_bytes(&self) -> u64 {
        self.config.limits.max_request_bytes
    }

    pub fn engine_status(&self) -> EngineStatus {
        self.engine.status()
    }

    pub fn origin_allowed(&self, origin: &str) -> bool {
        normalized_origin(origin).is_ok_and(|candidate| {
            self.config
                .allowed_origins
                .iter()
                .any(|allowed| normalized_origin(allowed).is_ok_and(|value| value == candidate))
        })
    }

    pub fn revoke(&self, token: &str) {
        if let Ok(mut sessions) = self.sessions.lock() {
            sessions.revoke(token);
        }
    }

    pub fn bounded_json(&self, value: &impl Serialize) -> Result<Vec<u8>, HostError> {
        let bytes = serde_json::to_vec(value)?;
        if bytes.len() as u64 > self.config.limits.max_response_bytes {
            return Err(HostError::Budget);
        }
        Ok(bytes)
    }
}

#[derive(Serialize)]
pub(super) struct EngineManifest {
    pub(super) kind: &'static str,
    pub(super) status: EngineStatus,
    pub(super) isolation: &'static str,
    pub(super) arbitrary_network: bool,
    pub(super) arbitrary_filesystem: bool,
}
