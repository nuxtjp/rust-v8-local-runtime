use nuxtjp_local_runtime_host::{EngineStatus, HostError, PayloadScalarKind, load_config};

const CONFIG: &str = "examples/local-simulation.json";

#[test]
fn accepts_closed_local_simulation() {
    let config = load_config(CONFIG).expect("closed configuration");
    assert_eq!(config.external_egress, "deny");
    assert!(!config.external_actions);
    assert_eq!(config.engine_status, EngineStatus::Simulated);
}

#[test]
fn rejects_remote_bind_and_unreviewed_engine_status() {
    let mut config = load_config(CONFIG).expect("fixture");
    config.bind = "0.0.0.0:37843".to_string();
    assert!(matches!(
        config.validate(),
        Err(HostError::Configuration(_))
    ));

    let mut config = load_config(CONFIG).expect("fixture");
    config.engine_status = EngineStatus::Ready;
    assert!(config.validate().is_err());
}

#[test]
fn rejects_external_action_and_ambiguous_origins() {
    let mut config = load_config(CONFIG).expect("fixture");
    config.external_actions = true;
    assert!(config.validate().is_err());

    let mut config = load_config(CONFIG).expect("fixture");
    config.allowed_origins = vec!["http://localhost:3000".to_string()];
    assert!(config.validate().is_err());
}

#[test]
fn rejects_conflicting_definitions_for_one_payload_schema_id() {
    let mut config = load_config(CONFIG).expect("fixture");
    config.capabilities[1].payload_schema.id = config.capabilities[0].payload_schema.id.clone();
    config.capabilities[1].payload_schema.fields[0].kind = PayloadScalarKind::Boolean;
    assert!(config.validate().is_err());
}

#[test]
fn rejects_payload_names_and_values_outside_browser_safe_bounds() {
    let mut config = load_config(CONFIG).expect("fixture");
    config.capabilities[0].payload_schema.fields[0].name = "Open-Count".to_string();
    assert!(config.validate().is_err());

    let mut config = load_config(CONFIG).expect("fixture");
    config.views[0].payload["open_count"] = serde_json::json!(9_007_199_254_740_992_u64);
    assert!(config.validate().is_err());
}

#[test]
fn rejects_unknown_configuration_and_unbounded_sessions() {
    let source = std::fs::read_to_string(CONFIG).expect("fixture");
    let changed = source.replacen(
        "\"runtime_id\":",
        "\"unexpected\": true, \"runtime_id\":",
        1,
    );
    assert!(serde_json::from_str::<nuxtjp_local_runtime_host::HostConfig>(&changed).is_err());

    let mut config = load_config(CONFIG).expect("fixture");
    config.limits.session_ttl_seconds = 3601;
    assert!(config.validate().is_err());
}
