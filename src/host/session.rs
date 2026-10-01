use super::LocalRuntimeHost;
use crate::{
    HostError, SessionGrant, SessionRequest,
    security::{SessionBudget, normalized_origin},
    session::SessionIssue,
};
use std::collections::HashSet;

impl LocalRuntimeHost {
    pub fn open_session(
        &self,
        header_origin: &str,
        request: SessionRequest,
        now: u64,
    ) -> Result<SessionGrant, HostError> {
        let origin = normalized_origin(header_origin)?;
        if request.schema != "nuxtjp://local-runtime/session-request/v1"
            || normalized_origin(&request.origin)? != origin
            || normalized_origin(&request.audience)? != origin
            || !self.origin_allowed(&origin)
        {
            return Err(HostError::Boundary);
        }
        let available: HashSet<&str> = self
            .config
            .capabilities
            .iter()
            .map(|item| item.id.as_str())
            .collect();
        let requested: HashSet<&String> = request.requested_capabilities.iter().collect();
        if request.requested_capabilities.len() > 64
            || request
                .requested_capabilities
                .iter()
                .any(|item| !available.contains(item.as_str()))
            || requested.len() != request.requested_capabilities.len()
        {
            return Err(HostError::Unauthorized);
        }
        let budget = SessionBudget {
            max_messages: self.config.limits.max_messages_per_minute,
            max_bytes: self.config.limits.max_bytes_per_minute,
        };
        self.sessions
            .lock()
            .map_err(|_| HostError::Unauthorized)?
            .issue(SessionIssue {
                subject_id: &self.config.subject_id,
                audience: &origin,
                client_nonce: &request.client_nonce,
                capabilities: request.requested_capabilities,
                now,
                ttl: self.config.limits.session_ttl_seconds,
                budget,
            })
    }
}
