use nuxtjp_local_runtime_host::{
    ContractSimulationEngine, EngineStatus, HostConfig, LocalRuntimeHost, ProcessViewEngine,
    SessionRequest, ViewEngine, ViewRequest, load_config, run_server,
};
use std::{env, process::ExitCode, sync::Arc};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or("command is required")?;
    let path = args.next().ok_or("configuration path is required")?;
    if args.next().is_some() {
        return Err("unexpected argument".into());
    }
    let config = load_config(path)?;
    if command == "validate-config" {
        println!(
            "{}",
            serde_json::json!({
                "schema": "nuxtjp://local-runtime/config-validation/v1",
                "runtime_id": config.runtime_id,
                "mode": config.mode,
                "engine_status": config.engine_status,
                "valid": true,
                "external_actions": false
            })
        );
        return Ok(());
    }
    let origin = config
        .allowed_origins
        .first()
        .cloned()
        .ok_or("allowed origin is required")?;
    let (capability, view) = primary_view(&config)?;
    let engine = build_engine(&config)?;
    let host = LocalRuntimeHost::new(config, engine)?;
    match command.as_str() {
        "serve" => run_server(host)?,
        "simulate" if host.engine_status() == EngineStatus::Simulated => {
            render_once(&host, &origin, &capability, &view)?
        }
        "render" if host.engine_status() == EngineStatus::Ready => {
            render_once(&host, &origin, &capability, &view)?
        }
        "simulate" | "render" => return Err("command does not match configured engine mode".into()),
        _ => return Err(format!("unknown command: {command}").into()),
    }
    Ok(())
}

fn build_engine(config: &HostConfig) -> Result<Arc<dyn ViewEngine>, Box<dyn std::error::Error>> {
    match config.engine_status {
        EngineStatus::Simulated => Ok(Arc::new(ContractSimulationEngine::new(&config.views))),
        EngineStatus::Ready => Ok(Arc::new(ProcessViewEngine::new(config)?)),
        EngineStatus::Disabled => Err("disabled engine cannot be started".into()),
    }
}

fn primary_view(config: &HostConfig) -> Result<(String, String), Box<dyn std::error::Error>> {
    if let Some(view) = config.views.first() {
        return Ok((view.capability_id.clone(), view.view_id.clone()));
    }
    let view = config
        .process_views
        .first()
        .ok_or("configured view is required")?;
    Ok((view.capability_id.clone(), view.view_id.clone()))
}

fn render_once(
    host: &LocalRuntimeHost,
    origin: &str,
    capability: &str,
    view: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let grant = host.open_session(
        origin,
        SessionRequest {
            schema: "nuxtjp://local-runtime/session-request/v1".to_string(),
            origin: origin.to_string(),
            audience: origin.to_string(),
            client_nonce: "simulation-client-nonce".to_string(),
            requested_capabilities: vec![capability.to_string()],
        },
        1,
    )?;
    let envelope = host.view(
        origin,
        &grant.token,
        ViewRequest {
            schema: "nuxtjp://local-runtime/view-request/v1".to_string(),
            capability_id: capability.to_string(),
            view_id: view.to_string(),
            request_nonce: "simulation-request-nonce".to_string(),
        },
        2,
    )?;
    println!("{}", serde_json::to_string_pretty(&envelope)?);
    Ok(())
}
