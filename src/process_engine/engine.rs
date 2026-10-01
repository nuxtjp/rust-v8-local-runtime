//! Maps reviewed service-card inputs to the worker contract and validates every response.

use super::{
    config::{ProcessEngineConfig, ProcessView},
    contract::{
        REQUEST_SCHEMA, RESPONSE_SCHEMA, SCRIPT_ID, SCRIPT_SHA256, WorkerRequest, WorkerResponse,
    },
    process,
};
use crate::{EngineStatus, HostConfig, HostError, InformationBand, RenderedView, ViewEngine};
use serde_json::json;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

pub struct ProcessViewEngine {
    config: ProcessEngineConfig,
    views: HashMap<(String, String), (ProcessView, String)>,
    sequence: AtomicU64,
}

impl ProcessViewEngine {
    pub fn new(host: &HostConfig) -> Result<Self, HostError> {
        host.validate()?;
        let config = host
            .process_engine
            .clone()
            .ok_or_else(|| invalid("process engine configuration is required"))?;
        let schemas = host
            .capabilities
            .iter()
            .map(|item| (item.id.as_str(), item.payload_schema.id.as_str()))
            .collect::<HashMap<_, _>>();
        let views = host
            .process_views
            .iter()
            .map(|view| {
                let schema = schemas
                    .get(view.capability_id.as_str())
                    .ok_or_else(|| invalid("process view capability is missing"))?;
                Ok((
                    (view.capability_id.clone(), view.view_id.clone()),
                    (view.clone(), (*schema).to_string()),
                ))
            })
            .collect::<Result<_, HostError>>()?;
        Ok(Self {
            config,
            views,
            sequence: AtomicU64::new(1),
        })
    }
}

impl ViewEngine for ProcessViewEngine {
    fn status(&self) -> EngineStatus {
        EngineStatus::Ready
    }

    fn render(&self, capability_id: &str, view_id: &str) -> Result<RenderedView, HostError> {
        let (view, payload_schema_id) = self
            .views
            .get(&(capability_id.to_string(), view_id.to_string()))
            .ok_or(HostError::Unavailable)?;
        let request_id = format!(
            "runtime-{:016x}",
            self.sequence.fetch_add(1, Ordering::Relaxed)
        );
        let request = WorkerRequest {
            schema: REQUEST_SCHEMA,
            request_id: &request_id,
            script_id: SCRIPT_ID,
            script_sha256: SCRIPT_SHA256,
            limits: &self.config.limits,
            payload: &view.payload,
        };
        let encoded = serde_json::to_vec(&request)?;
        let output = process::invoke(&self.config, &encoded)?;
        let response: WorkerResponse =
            serde_json::from_slice(&output).map_err(|_| HostError::Unavailable)?;
        validate_response(&request_id, &view.payload, &response)?;
        Ok(RenderedView {
            band: InformationBand::Session,
            payload_schema_id: payload_schema_id.clone(),
            payload: json!({
                "title": response.view.title,
                "summary": response.view.summary,
                "status_label": response.view.status_label,
                "highlighted": response.view.highlighted,
                "html": response.view.html,
            }),
        })
    }
}

fn validate_response(
    request_id: &str,
    input: &super::contract::WorkerPayload,
    response: &WorkerResponse,
) -> Result<(), HostError> {
    let view = &response.view;
    if response.schema != RESPONSE_SCHEMA
        || response.request_id != request_id
        || response.status != "rendered"
        || view.title != input.title
        || view.summary != input.summary
        || view.status_label != input.status_label
        || view.highlighted != input.highlighted
        || view.html.is_empty()
        || view.html.len() > 4_096
    {
        return Err(HostError::Unavailable);
    }
    Ok(())
}

fn invalid(message: &str) -> HostError {
    HostError::Configuration(message.into())
}
