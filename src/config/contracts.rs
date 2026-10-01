//! Correlates every configured view with one exact browser-safe capability schema.

use super::HostConfig;
use crate::{HostError, InformationBand, PayloadScalarKind, security::normalized_origin};
use std::collections::{HashMap, HashSet};

impl HostConfig {
    pub(super) fn validate_contracts(&self) -> Result<(), HostError> {
        let origins: HashSet<String> = self
            .allowed_origins
            .iter()
            .map(|origin| normalized_origin(origin))
            .collect::<Result<_, _>>()?;
        if origins.len() != self.allowed_origins.len() {
            return Err(invalid("allowed origins must be unique"));
        }
        let capabilities: HashMap<_, _> = self
            .capabilities
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();
        if capabilities.len() != self.capabilities.len()
            || self.capabilities.iter().any(|item| {
                item.id.trim().is_empty()
                    || item.service_id.trim().is_empty()
                    || item.description.trim().is_empty()
                    || item.output_band != InformationBand::Session
                    || item.payload_schema.validate().is_err()
            })
        {
            return Err(invalid("capability contract is invalid"));
        }
        self.validate_simulation_views(&capabilities)?;
        self.validate_process_views(&capabilities)?;
        validate_schema_identity(&self.capabilities)
    }

    fn validate_simulation_views(
        &self,
        capabilities: &HashMap<&str, &crate::RuntimeCapability>,
    ) -> Result<(), HostError> {
        if self.views.iter().any(|view| {
            capabilities
                .get(view.capability_id.as_str())
                .is_none_or(|capability| {
                    view.view_id.trim().is_empty()
                        || view.band != capability.output_band
                        || view.payload_schema_id != capability.payload_schema.id
                        || capability
                            .payload_schema
                            .validate_payload(&view.payload)
                            .is_err()
                })
        }) {
            return Err(invalid("simulation view contract is invalid"));
        }
        unique_pairs(
            self.views
                .iter()
                .map(|view| (view.capability_id.as_str(), view.view_id.as_str())),
        )
    }

    fn validate_process_views(
        &self,
        capabilities: &HashMap<&str, &crate::RuntimeCapability>,
    ) -> Result<(), HostError> {
        if self.process_views.iter().any(|view| {
            capabilities
                .get(view.capability_id.as_str())
                .is_none_or(|capability| {
                    view.view_id.trim().is_empty()
                        || !service_card_schema(&capability.payload_schema)
                        || view.payload.validate().is_err()
                })
        }) {
            return Err(invalid("process view contract is invalid"));
        }
        unique_pairs(
            self.process_views
                .iter()
                .map(|view| (view.capability_id.as_str(), view.view_id.as_str())),
        )
    }
}

fn validate_schema_identity(capabilities: &[crate::RuntimeCapability]) -> Result<(), HostError> {
    let mut schemas = HashMap::new();
    for capability in capabilities {
        if let Some(existing) =
            schemas.insert(&capability.payload_schema.id, &capability.payload_schema)
            && existing != &capability.payload_schema
        {
            return Err(invalid("one schema id cannot describe different fields"));
        }
    }
    Ok(())
}

fn unique_pairs<'a>(pairs: impl Iterator<Item = (&'a str, &'a str)>) -> Result<(), HostError> {
    let items = pairs.collect::<Vec<_>>();
    let unique = items.iter().copied().collect::<HashSet<_>>();
    (unique.len() == items.len())
        .then_some(())
        .ok_or_else(|| invalid("capability view pairs must be unique"))
}

fn service_card_schema(schema: &crate::PayloadSchema) -> bool {
    let actual = schema
        .fields
        .iter()
        .map(|field| (field.name.as_str(), field.kind))
        .collect::<HashMap<_, _>>();
    actual.len() == 5
        && actual.get("title") == Some(&PayloadScalarKind::Text)
        && actual.get("summary") == Some(&PayloadScalarKind::Text)
        && actual.get("status_label") == Some(&PayloadScalarKind::Text)
        && actual.get("highlighted") == Some(&PayloadScalarKind::Boolean)
        && actual.get("html") == Some(&PayloadScalarKind::Text)
}

fn invalid(message: &str) -> HostError {
    HostError::Configuration(message.to_string())
}
