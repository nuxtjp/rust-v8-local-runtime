use super::LocalRuntimeHost;
use crate::{
    BoundaryTarget, HostError, ViewEnvelope, ViewRequest, band_allows_target,
    security::{browser_visible, normalized_origin},
};

impl LocalRuntimeHost {
    pub fn view(
        &self,
        header_origin: &str,
        token: &str,
        request: ViewRequest,
        now: u64,
    ) -> Result<ViewEnvelope, HostError> {
        let origin = normalized_origin(header_origin)?;
        if request.schema != "nuxtjp://local-runtime/view-request/v1"
            || !self.origin_allowed(&origin)
        {
            return Err(HostError::Boundary);
        }
        let mut sessions = self.sessions.lock().map_err(|_| HostError::Unauthorized)?;
        sessions.authorize(
            token,
            &origin,
            &request.capability_id,
            &request.request_nonce,
            now,
        )?;
        let rendered = self
            .engine
            .render(&request.capability_id, &request.view_id)?;
        let capability = self
            .config
            .capabilities
            .iter()
            .find(|item| item.id == request.capability_id)
            .ok_or(HostError::Unauthorized)?;
        if !browser_visible(rendered.band)
            || !band_allows_target(rendered.band, BoundaryTarget::LoopbackBrowser)
            || rendered.band != capability.output_band
            || rendered.payload_schema_id != capability.payload_schema.id
        {
            return Err(HostError::Boundary);
        }
        capability
            .payload_schema
            .validate_payload(&rendered.payload)
            .map_err(|_| HostError::Boundary)?;
        let envelope = ViewEnvelope {
            schema: "nuxtjp://local-runtime/view/v1",
            capability_id: request.capability_id,
            view_id: request.view_id,
            band: rendered.band,
            payload_schema_id: rendered.payload_schema_id,
            generated_at_unix_seconds: now,
            expires_at_unix_seconds: now.saturating_add(30),
            external_actions: false,
            payload: rendered.payload,
        };
        let encoded = self.bounded_json(&envelope)?;
        sessions.consume(token, encoded.len(), now)?;
        Ok(envelope)
    }
}
