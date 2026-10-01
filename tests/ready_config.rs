use nuxtjp_local_runtime_host::{
    EngineStatus, InformationBand, PayloadField, PayloadScalarKind, ProcessEngineConfig,
    ProcessView, RuntimeMode, WorkerLimits, WorkerPayload, load_config, worker_sha256,
};

const CONFIG: &str = "examples/local-simulation.json";

#[test]
fn accepts_only_a_pinned_ready_process_configuration() {
    let mut config = ready_config();
    assert!(config.validate().is_ok());
    config
        .process_engine
        .as_mut()
        .expect("process engine")
        .worker_sha256 = "0".repeat(64);
    assert!(config.validate().is_err());

    let mut config = ready_config();
    config.mode = RuntimeMode::LocalSimulation;
    assert!(config.validate().is_err());
}

#[cfg(unix)]
#[test]
fn rejects_a_worker_path_reached_through_a_symlink() {
    let mut config = ready_config();
    let target = std::env::current_exe().expect("test executable");
    let link = std::env::temp_dir().join(format!(
        "nuxtjp-runtime-worker-link-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    std::os::unix::fs::symlink(target, &link).expect("temporary symlink");
    config
        .process_engine
        .as_mut()
        .expect("process engine")
        .worker_path = link.to_string_lossy().to_string();
    assert!(config.validate().is_err());
    std::fs::remove_file(link).expect("remove temporary symlink");
}

fn ready_config() -> nuxtjp_local_runtime_host::HostConfig {
    let mut config = load_config(CONFIG).expect("simulation fixture");
    let worker = std::env::current_exe()
        .expect("test executable")
        .canonicalize()
        .expect("canonical executable");
    config.mode = RuntimeMode::LocalProduction;
    config.engine_status = EngineStatus::Ready;
    config.views.clear();
    config.capabilities.truncate(1);
    config.capabilities[0].id = "nuxtjp.service-card.render".into();
    config.capabilities[0].payload_schema.id = "nuxtjp.service-card.v1".into();
    config.capabilities[0].payload_schema.fields = service_card_fields();
    config.process_engine = Some(ProcessEngineConfig {
        schema: "nuxtjp://local-runtime/process-engine/v1".into(),
        worker_path: worker.to_string_lossy().to_string(),
        worker_sha256: worker_sha256(&worker).expect("worker digest"),
        expires_unix_seconds: u64::MAX,
        script_id: "nuxtjp-service-card-v1".into(),
        script_sha256: "bd954b068b08f679b2c90c788b907100a184097693bbf24ddd119937d4119b8e".into(),
        limits: WorkerLimits {
            input_bytes: 65_536,
            output_bytes: 65_536,
            heap_mib: 64,
            timeout_ms: 2_000,
        },
    });
    config.process_views = vec![ProcessView {
        capability_id: "nuxtjp.service-card.render".into(),
        view_id: "runtime-status".into(),
        payload: WorkerPayload {
            title: "Runtime".into(),
            summary: "Pinned worker".into(),
            status_label: "Local".into(),
            highlighted: true,
        },
    }];
    config.capabilities[0].output_band = InformationBand::Session;
    config
}

fn service_card_fields() -> Vec<PayloadField> {
    vec![
        field("title", PayloadScalarKind::Text),
        field("summary", PayloadScalarKind::Text),
        field("status_label", PayloadScalarKind::Text),
        field("highlighted", PayloadScalarKind::Boolean),
        field("html", PayloadScalarKind::Text),
    ]
}

fn field(name: &str, kind: PayloadScalarKind) -> PayloadField {
    PayloadField {
        name: name.into(),
        kind,
    }
}
