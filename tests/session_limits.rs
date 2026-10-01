use nuxtjp_local_runtime_host::{
    ContractSimulationEngine, HostError, LocalRuntimeHost, SessionRequest, load_config,
};
use std::sync::Arc;

const CONFIG: &str = "examples/local-simulation.json";
const ORIGIN: &str = "https://nerp.jp";

fn host() -> LocalRuntimeHost {
    let config = load_config(CONFIG).expect("fixture");
    let engine = Arc::new(ContractSimulationEngine::new(&config.views));
    LocalRuntimeHost::new(config, engine).expect("host")
}

fn request(index: usize) -> SessionRequest {
    SessionRequest {
        schema: "nuxtjp://local-runtime/session-request/v1".to_string(),
        origin: ORIGIN.to_string(),
        audience: ORIGIN.to_string(),
        client_nonce: format!("bounded-session-{index:04}"),
        requested_capabilities: vec!["workflow.issue.read".to_string()],
    }
}

#[test]
fn bounds_active_sessions_and_recent_pairing_nonces() {
    let active_host = host();
    for index in 0..64 {
        active_host
            .open_session(ORIGIN, request(index), 10)
            .expect("bounded active session");
    }
    assert!(matches!(
        active_host.open_session(ORIGIN, request(64), 10),
        Err(HostError::Budget)
    ));

    let nonce_host = host();
    for index in 0..256 {
        let grant = nonce_host
            .open_session(ORIGIN, request(index), 10)
            .expect("bounded recent nonce");
        nonce_host.revoke(&grant.token);
    }
    assert!(matches!(
        nonce_host.open_session(ORIGIN, request(256), 10),
        Err(HostError::Budget)
    ));
}
