use nuxtjp_local_runtime_host::{
    BoundaryTarget, ContractSimulationEngine, HostError, InformationBand, LocalRuntimeHost,
    SessionRequest, ViewRequest, band_allows_target, load_config,
};
use std::sync::Arc;

const CONFIG: &str = "examples/local-simulation.json";
const ORIGIN: &str = "https://nerp.jp";
const CAPABILITY: &str = "workflow.issue.read";

fn host() -> LocalRuntimeHost {
    let config = load_config(CONFIG).expect("fixture");
    let engine = Arc::new(ContractSimulationEngine::new(&config.views));
    LocalRuntimeHost::new(config, engine).expect("host")
}

fn session_request(nonce: &str) -> SessionRequest {
    SessionRequest {
        schema: "nuxtjp://local-runtime/session-request/v1".to_string(),
        origin: ORIGIN.to_string(),
        audience: ORIGIN.to_string(),
        client_nonce: nonce.to_string(),
        requested_capabilities: vec![CAPABILITY.to_string()],
    }
}

fn view_request(nonce: &str) -> ViewRequest {
    ViewRequest {
        schema: "nuxtjp://local-runtime/view-request/v1".to_string(),
        capability_id: CAPABILITY.to_string(),
        view_id: "issue-summary".to_string(),
        request_nonce: nonce.to_string(),
    }
}

#[test]
fn pairs_and_returns_session_only_projection() {
    let host = host();
    let grant = host
        .open_session(ORIGIN, session_request("client-nonce-one"), 10)
        .expect("session");
    let view = host
        .view(ORIGIN, &grant.token, view_request("request-nonce-one"), 11)
        .expect("view");
    assert_eq!(view.band, InformationBand::Session);
    assert_eq!(view.payload_schema_id, "nuxtjp.workflow-issue-summary.v1");
    assert_eq!(view.payload["source"], "local-simulation");
    assert!(!view.external_actions);
}

#[test]
fn rejects_origin_capability_and_nonce_reuse_without_oracle() {
    let host = host();
    assert!(matches!(
        host.open_session(
            "https://attacker.example",
            session_request("client-nonce-two"),
            10
        ),
        Err(HostError::Boundary)
    ));
    let grant = host
        .open_session(ORIGIN, session_request("client-nonce-three"), 10)
        .expect("session");
    host.view(ORIGIN, &grant.token, view_request("request-nonce-two"), 11)
        .expect("first use");
    assert!(matches!(
        host.view(ORIGIN, &grant.token, view_request("request-nonce-two"), 12),
        Err(HostError::Replay)
    ));
    let mut unknown = view_request("request-nonce-three");
    unknown.capability_id = "unknown.read".to_string();
    assert!(matches!(
        host.view(ORIGIN, &grant.token, unknown, 12),
        Err(HostError::Unauthorized)
    ));
}

#[test]
fn information_bands_do_not_implicitly_declassify() {
    assert!(band_allows_target(
        InformationBand::Session,
        BoundaryTarget::LoopbackBrowser
    ));
    assert!(!band_allows_target(
        InformationBand::Local,
        BoundaryTarget::LoopbackBrowser
    ));
    assert!(!band_allows_target(
        InformationBand::Session,
        BoundaryTarget::ExternalControl
    ));
}

#[test]
fn works_without_hatter_configuration_or_environment() {
    let host = host();
    let manifest = host.manifest_json().expect("manifest");
    let text = String::from_utf8(manifest).expect("UTF-8");
    assert!(!text.to_lowercase().contains("hatter"));
    assert!(text.contains("\"external_egress\":\"deny\""));
}

#[test]
fn enforces_per_session_minute_budget() {
    let mut config = load_config(CONFIG).expect("fixture");
    config.limits.max_messages_per_minute = 1;
    let engine = Arc::new(ContractSimulationEngine::new(&config.views));
    let host = LocalRuntimeHost::new(config, engine).expect("host");
    let grant = host
        .open_session(ORIGIN, session_request("client-nonce-budget"), 10)
        .expect("session");
    host.view(
        ORIGIN,
        &grant.token,
        view_request("request-nonce-budget-a"),
        11,
    )
    .expect("first view");
    assert!(matches!(
        host.view(
            ORIGIN,
            &grant.token,
            view_request("request-nonce-budget-b"),
            12
        ),
        Err(HostError::Budget)
    ));
}
