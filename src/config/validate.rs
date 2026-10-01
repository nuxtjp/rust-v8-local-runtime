//! Enforces mutually exclusive simulation and ready process-engine modes.

use super::HostConfig;
use crate::{EngineStatus, HostError, RuntimeMode};
use std::net::SocketAddr;

impl HostConfig {
    pub fn validate(&self) -> Result<(), HostError> {
        let bind = self
            .bind
            .parse::<SocketAddr>()
            .map_err(|_| invalid("bind must be a numeric socket address"))?;
        if !bind.ip().is_loopback()
            || !bounded_text(&self.runtime_id, 128)
            || !bounded_text(&self.subject_id, 128)
            || self.external_actions
            || self.external_egress != "deny"
            || self.allowed_origins.is_empty()
            || self.allowed_origins.len() > 16
            || self.capabilities.len() > 64
        {
            return Err(invalid("closed runtime settings are required"));
        }
        self.validate_limits()?;
        self.validate_engine_mode()?;
        self.validate_contracts()
    }

    fn validate_engine_mode(&self) -> Result<(), HostError> {
        match (self.mode, self.engine_status, &self.process_engine) {
            (RuntimeMode::LocalSimulation, EngineStatus::Simulated, None)
                if self.process_views.is_empty() && !self.views.is_empty() =>
            {
                Ok(())
            }
            (RuntimeMode::LocalProduction, EngineStatus::Ready, Some(engine))
                if self.views.is_empty() && !self.process_views.is_empty() =>
            {
                engine.validate()
            }
            _ => Err(invalid(
                "simulation fixtures and ready process settings must remain exclusive",
            )),
        }
    }

    fn validate_limits(&self) -> Result<(), HostError> {
        let limits = &self.limits;
        if limits.max_request_bytes == 0
            || limits.max_response_bytes == 0
            || limits.max_messages_per_minute == 0
            || limits.max_bytes_per_minute == 0
            || limits.session_ttl_seconds == 0
            || limits.max_request_bytes > 1024 * 1024
            || limits.max_response_bytes > 8 * 1024 * 1024
            || limits.max_messages_per_minute > 600
            || limits.max_bytes_per_minute > 64 * 1024 * 1024
            || limits.session_ttl_seconds > 3600
        {
            return Err(invalid("runtime limits are outside accepted bounds"));
        }
        Ok(())
    }
}

fn bounded_text(value: &str, maximum_bytes: usize) -> bool {
    !value.trim().is_empty() && value.len() <= maximum_bytes
}

fn invalid(message: &str) -> HostError {
    HostError::Configuration(message.to_string())
}
