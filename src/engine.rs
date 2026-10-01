use crate::{EngineStatus, HostError, RenderedView, ViewFixture};
use std::collections::HashMap;

pub trait ViewEngine: Send + Sync {
    fn status(&self) -> EngineStatus;
    fn render(&self, capability_id: &str, view_id: &str) -> Result<RenderedView, HostError>;
}

/// Contract-only engine used before a separately sandboxed V8 adapter exists.
pub struct ContractSimulationEngine {
    views: HashMap<(String, String), RenderedView>,
}

impl ContractSimulationEngine {
    pub fn new(fixtures: &[ViewFixture]) -> Self {
        let views = fixtures
            .iter()
            .map(|fixture| {
                (
                    (fixture.capability_id.clone(), fixture.view_id.clone()),
                    RenderedView {
                        band: fixture.band,
                        payload_schema_id: fixture.payload_schema_id.clone(),
                        payload: fixture.payload.clone(),
                    },
                )
            })
            .collect();
        Self { views }
    }
}

impl ViewEngine for ContractSimulationEngine {
    fn status(&self) -> EngineStatus {
        EngineStatus::Simulated
    }

    fn render(&self, capability_id: &str, view_id: &str) -> Result<RenderedView, HostError> {
        self.views
            .get(&(capability_id.to_string(), view_id.to_string()))
            .cloned()
            .ok_or(HostError::Unavailable)
    }
}
