use nuxtjp_local_runtime_host::{
    EngineStatus, HostError, LocalRuntimeHost, RenderedView, SessionRequest, ViewEngine,
    ViewRequest, load_config,
};
use serde_json::json;
use std::sync::Arc;

const CONFIG: &str = "examples/local-simulation.json";
const ORIGIN: &str = "https://nerp.jp";
const CAPABILITY: &str = "workflow.issue.read";
const SCHEMA_ID: &str = "nuxtjp.workflow-issue-summary.v1";

struct FixedEngine(RenderedView);

impl ViewEngine for FixedEngine {
    fn status(&self) -> EngineStatus {
        EngineStatus::Simulated
    }

    fn render(&self, _: &str, _: &str) -> Result<RenderedView, HostError> {
        Ok(self.0.clone())
    }
}

fn open_with(result: RenderedView) -> (LocalRuntimeHost, String) {
    let config = load_config(CONFIG).expect("fixture");
    let host = LocalRuntimeHost::new(config, Arc::new(FixedEngine(result))).expect("host");
    let grant = host
        .open_session(
            ORIGIN,
            SessionRequest {
                schema: "nuxtjp://local-runtime/session-request/v1".to_string(),
                origin: ORIGIN.to_string(),
                audience: ORIGIN.to_string(),
                client_nonce: "payload-contract-client".to_string(),
                requested_capabilities: vec![CAPABILITY.to_string()],
            },
            10,
        )
        .expect("session");
    (host, grant.token)
}

fn request() -> ViewRequest {
    ViewRequest {
        schema: "nuxtjp://local-runtime/view-request/v1".to_string(),
        capability_id: CAPABILITY.to_string(),
        view_id: "issue-summary".to_string(),
        request_nonce: "payload-contract-request".to_string(),
    }
}

fn valid_result() -> RenderedView {
    RenderedView {
        band: nuxtjp_local_runtime_host::InformationBand::Session,
        payload_schema_id: SCHEMA_ID.to_string(),
        payload: json!({
            "open_count": 2,
            "blocked_count": 1,
            "source": "simulation",
            "local_only": true
        }),
    }
}

#[test]
fn rejects_invalid_configured_payloads() {
    let mut extra = load_config(CONFIG).expect("fixture");
    extra.views[0].payload["unexpected"] = json!(true);
    assert!(extra.validate().is_err());

    let mut wrong_kind = load_config(CONFIG).expect("fixture");
    wrong_kind.views[0].payload["open_count"] = json!("two");
    assert!(wrong_kind.validate().is_err());

    let mut missing = load_config(CONFIG).expect("fixture");
    missing.views[0]
        .payload
        .as_object_mut()
        .expect("object")
        .remove("local_only");
    assert!(missing.validate().is_err());

    let mut wrong_schema = load_config(CONFIG).expect("fixture");
    wrong_schema.views[0].payload_schema_id = "unknown.schema.v1".to_string();
    assert!(wrong_schema.validate().is_err());
}

#[test]
fn rejects_engine_schema_mismatch_before_returning() {
    let mut result = valid_result();
    result.payload_schema_id = "unknown.schema.v1".to_string();
    let (host, token) = open_with(result);
    assert!(matches!(
        host.view(ORIGIN, &token, request(), 11),
        Err(HostError::Boundary)
    ));
}

#[test]
fn rejects_engine_key_and_scalar_changes_before_returning() {
    let mut extra = valid_result();
    extra.payload["private_detail"] = json!("must not cross");
    let (host, token) = open_with(extra);
    assert!(matches!(
        host.view(ORIGIN, &token, request(), 11),
        Err(HostError::Boundary)
    ));

    let mut nested = valid_result();
    nested.payload["source"] = json!({"nested": "not allowed"});
    let (host, token) = open_with(nested);
    assert!(matches!(
        host.view(ORIGIN, &token, request(), 11),
        Err(HostError::Boundary)
    ));
}
