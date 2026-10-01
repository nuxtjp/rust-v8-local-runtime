use super::{body, boundary, response};
use crate::{LocalRuntimeHost, SessionRequest, ViewRequest};
use serde_json::json;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use tiny_http::{Method, Request};

pub(crate) fn handle(request: Request, host: &Arc<LocalRuntimeHost>) {
    if let Err(error) = boundary::validate_loopback(&request, host) {
        response::error(request, &error, None);
        return;
    }
    let path = request.url().split('?').next().unwrap_or(request.url());
    let method = request.method().clone();
    let origin = boundary::header(&request, "Origin").map(str::to_string);
    let result = match (method, path) {
        (Method::Get, "/v1/manifest") => manifest(request, host, origin.as_deref()),
        (Method::Options, _) => preflight(request, host),
        (Method::Post, "/v1/session") => session(request, host),
        (Method::Post, "/v1/view") => view(request, host),
        (Method::Delete, "/v1/session") => revoke(request, host),
        _ => {
            response::json(request, 404, &json!({ "error": "not found" }), None);
            Ok(())
        }
    };
    if let Err(route_error) = result {
        let (request, error, origin) = *route_error;
        response::error(request, &error, origin.as_deref());
    }
}

type RouteError = Box<(Request, crate::HostError, Option<String>)>;
type RouteResult = Result<(), RouteError>;

fn failed(request: Request, error: crate::HostError, origin: Option<String>) -> RouteError {
    Box::new((request, error, origin))
}

fn manifest(request: Request, host: &LocalRuntimeHost, origin: Option<&str>) -> RouteResult {
    if origin.is_some_and(|value| !host.origin_allowed(value)) {
        return Err(failed(request, crate::HostError::Boundary, None));
    }
    match host.manifest_json() {
        Ok(bytes) => {
            let value: serde_json::Value = match serde_json::from_slice(&bytes) {
                Ok(value) => value,
                Err(error) => {
                    return Err(failed(request, error.into(), origin.map(str::to_string)));
                }
            };
            response::json(request, 200, &value, origin);
            Ok(())
        }
        Err(error) => Err(failed(request, error, origin.map(str::to_string))),
    }
}

fn preflight(request: Request, host: &LocalRuntimeHost) -> RouteResult {
    match boundary::required_origin(&request, host) {
        Ok(origin) => {
            let origin = origin.to_string();
            response::empty(request, 204, &origin);
            Ok(())
        }
        Err(error) => Err(failed(request, error, None)),
    }
}

fn session(mut request: Request, host: &LocalRuntimeHost) -> RouteResult {
    let origin = match boundary::required_origin(&request, host) {
        Ok(value) => value.to_string(),
        Err(error) => return Err(failed(request, error, None)),
    };
    if let Err(error) = boundary::require_client_marker(&request) {
        return Err(failed(request, error, Some(origin)));
    }
    if let Err(error) = boundary::require_json(&request) {
        return Err(failed(request, error, Some(origin)));
    }
    let input: SessionRequest = match body::read_json(&mut request, host.maximum_request_bytes()) {
        Ok(input) => input,
        Err(error) => return Err(failed(request, error, Some(origin))),
    };
    match host.open_session(&origin, input, now()) {
        Ok(grant) => {
            response::json(request, 200, &grant, Some(&origin));
            Ok(())
        }
        Err(error) => Err(failed(request, error, Some(origin))),
    }
}

fn view(mut request: Request, host: &LocalRuntimeHost) -> RouteResult {
    let origin = match boundary::required_origin(&request, host) {
        Ok(value) => value.to_string(),
        Err(error) => return Err(failed(request, error, None)),
    };
    if let Err(error) = boundary::require_client_marker(&request) {
        return Err(failed(request, error, Some(origin)));
    }
    let token = match boundary::bearer(&request) {
        Ok(value) => value.to_string(),
        Err(error) => return Err(failed(request, error, Some(origin))),
    };
    if let Err(error) = boundary::require_json(&request) {
        return Err(failed(request, error, Some(origin)));
    }
    let input: ViewRequest = match body::read_json(&mut request, host.maximum_request_bytes()) {
        Ok(input) => input,
        Err(error) => return Err(failed(request, error, Some(origin))),
    };
    match host.view(&origin, &token, input, now()) {
        Ok(envelope) => {
            response::json(request, 200, &envelope, Some(&origin));
            Ok(())
        }
        Err(error) => Err(failed(request, error, Some(origin))),
    }
}

fn revoke(request: Request, host: &LocalRuntimeHost) -> RouteResult {
    let origin = match boundary::required_origin(&request, host) {
        Ok(value) => value.to_string(),
        Err(error) => return Err(failed(request, error, None)),
    };
    if let Err(error) = boundary::require_client_marker(&request) {
        return Err(failed(request, error, Some(origin)));
    }
    let token = match boundary::bearer(&request) {
        Ok(value) => value.to_string(),
        Err(error) => return Err(failed(request, error, Some(origin))),
    };
    host.revoke(&token);
    response::json(request, 200, &json!({ "revoked": true }), Some(&origin));
    Ok(())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}
