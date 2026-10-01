use crate::HostError;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct SessionBudget {
    pub max_messages: u64,
    pub max_bytes: u64,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct BudgetUsage {
    budget: SessionBudget,
    window_started_at: u64,
    messages: u64,
    bytes: u64,
}

impl BudgetUsage {
    pub(crate) fn new(budget: SessionBudget, now: u64) -> Self {
        Self {
            budget,
            window_started_at: now,
            messages: 0,
            bytes: 0,
        }
    }

    pub(crate) fn consume(&mut self, bytes: usize, now: u64) -> Result<(), HostError> {
        if now >= self.window_started_at.saturating_add(60) {
            self.window_started_at = now;
            self.messages = 0;
            self.bytes = 0;
        }
        let bytes = u64::try_from(bytes).map_err(|_| HostError::Budget)?;
        if self.messages.saturating_add(1) > self.budget.max_messages
            || self.bytes.saturating_add(bytes) > self.budget.max_bytes
        {
            return Err(HostError::Budget);
        }
        self.messages += 1;
        self.bytes += bytes;
        Ok(())
    }
}
