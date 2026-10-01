use crate::HostError;
use serde::Serialize;
use tiny_http::{Header, Request, Response, StatusCode};

pub(crate) fn json(request: Request, status: u16, value: &impl Serialize, origin: Option<&str>) {
    let data = serde_json::to_vec(value)
        .unwrap_or_else(|_| br#"{"error":"response serialization failed"}"#.to_vec());
    let mut response = Response::from_data(data)
        .with_status_code(StatusCode(status))
        .with_header(header("Content-Type", "application/json"))
        .with_header(header("Cache-Control", "no-store"))
        .with_header(header("X-Content-Type-Options", "nosniff"))
        .with_header(header("Vary", "Origin"));
    if let Some(origin) = origin {
        response.add_header(header("Access-Control-Allow-Origin", origin));
    }
    let _ = request.respond(response);
}

pub(crate) fn empty(request: Request, status: u16, origin: &str) {
    let response = Response::empty(StatusCode(status))
        .with_header(header("Access-Control-Allow-Origin", origin))
        .with_header(header(
            "Access-Control-Allow-Methods",
            "GET, POST, DELETE, OPTIONS",
        ))
        .with_header(header(
            "Access-Control-Allow-Headers",
            "Authorization, Content-Type, X-NuxtJP-Local-Runtime",
        ))
        .with_header(header("Access-Control-Allow-Private-Network", "true"))
        .with_header(header("Access-Control-Max-Age", "300"))
        .with_header(header("Vary", "Origin"));
    let _ = request.respond(response);
}

pub(crate) fn error(request: Request, error: &HostError, origin: Option<&str>) {
    let status = match error {
        HostError::Boundary => 403,
        HostError::Unauthorized | HostError::Unavailable => 404,
        HostError::Replay => 409,
        HostError::Budget => 429,
        HostError::Request(_) | HostError::Json(_) => 400,
        HostError::Configuration(_) | HostError::Io(_) => 500,
    };
    let message = match error {
        HostError::Unauthorized | HostError::Unavailable => "not authorized",
        _ => "request rejected",
    };
    json(
        request,
        status,
        &serde_json::json!({ "error": message }),
        origin,
    );
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name, value).expect("static response header must be valid")
}
