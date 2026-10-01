mod body;
mod boundary;
mod response;
mod router;

use crate::{HostError, LocalRuntimeHost};
use std::sync::Arc;
use tiny_http::Server;

pub fn run_server(host: LocalRuntimeHost) -> Result<(), HostError> {
    let bind = host.bind().to_string();
    let server = Server::http(&bind)
        .map_err(|error| HostError::Configuration(format!("listener failed: {error}")))?;
    let host = Arc::new(host);
    println!("NuxtJP local runtime: http://{bind}");
    println!("External egress: denied; engine: simulated");
    for request in server.incoming_requests() {
        router::handle(request, &host);
    }
    Ok(())
}
