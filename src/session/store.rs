use super::token::random_token;
use crate::{
    HostError, InformationBand, SessionGrant,
    security::{BudgetUsage, SessionBudget},
};
use std::collections::{HashMap, HashSet};

const MAX_ACTIVE_SESSIONS: usize = 64;
const MAX_SESSION_NONCES: usize = 256;

pub(crate) struct SessionStore {
    sessions: HashMap<String, Session>,
    client_nonces: HashMap<String, u64>,
}

struct Session {
    audience: String,
    expires_at: u64,
    capabilities: HashSet<String>,
    request_nonces: HashSet<String>,
    usage: BudgetUsage,
}

pub(crate) struct SessionIssue<'a> {
    pub(crate) subject_id: &'a str,
    pub(crate) audience: &'a str,
    pub(crate) client_nonce: &'a str,
    pub(crate) capabilities: Vec<String>,
    pub(crate) now: u64,
    pub(crate) ttl: u64,
    pub(crate) budget: SessionBudget,
}

impl SessionStore {
    pub(crate) fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            client_nonces: HashMap::new(),
        }
    }

    pub(crate) fn issue(&mut self, input: SessionIssue<'_>) -> Result<SessionGrant, HostError> {
        self.sessions
            .retain(|_, session| session.expires_at >= input.now);
        self.client_nonces
            .retain(|_, expires_at| *expires_at >= input.now);
        if self.sessions.len() >= MAX_ACTIVE_SESSIONS
            || self.client_nonces.len() >= MAX_SESSION_NONCES
        {
            return Err(HostError::Budget);
        }
        if !(8..=128).contains(&input.client_nonce.len())
            || self.client_nonces.contains_key(input.client_nonce)
        {
            return Err(HostError::Replay);
        }
        let token = random_token();
        let session_id = random_token();
        let expires_at = input.now.saturating_add(input.ttl);
        self.client_nonces
            .insert(input.client_nonce.to_string(), expires_at);
        let session = Session {
            audience: input.audience.to_string(),
            expires_at,
            capabilities: input.capabilities.iter().cloned().collect(),
            request_nonces: HashSet::new(),
            usage: BudgetUsage::new(input.budget, input.now),
        };
        self.sessions.insert(token.clone(), session);
        Ok(SessionGrant {
            schema: "nuxtjp://local-runtime/session-grant/v1",
            session_id,
            token,
            subject_id: input.subject_id.to_string(),
            audience: input.audience.to_string(),
            expires_at_unix_seconds: expires_at,
            granted_capabilities: input.capabilities,
            classification_ceiling: InformationBand::Session,
            budget: input.budget,
            external_actions: false,
        })
    }

    pub(crate) fn authorize(
        &mut self,
        token: &str,
        audience: &str,
        capability: &str,
        nonce: &str,
        now: u64,
    ) -> Result<(), HostError> {
        let session = self
            .sessions
            .get_mut(token)
            .ok_or(HostError::Unauthorized)?;
        if session.expires_at < now
            || session.audience != audience
            || !session.capabilities.contains(capability)
        {
            return Err(HostError::Unauthorized);
        }
        if !(8..=128).contains(&nonce.len()) || !session.request_nonces.insert(nonce.to_string()) {
            return Err(HostError::Replay);
        }
        Ok(())
    }

    pub(crate) fn consume(&mut self, token: &str, bytes: usize, now: u64) -> Result<(), HostError> {
        self.sessions
            .get_mut(token)
            .ok_or(HostError::Unauthorized)?
            .usage
            .consume(bytes, now)
    }

    pub(crate) fn revoke(&mut self, token: &str) {
        self.sessions.remove(token);
    }
}
