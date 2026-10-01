use crate::{HostError, LocalRuntimeHost};
use std::net::SocketAddr;
use tiny_http::Request;

pub(crate) fn validate_loopback(
    request: &Request,
    host: &LocalRuntimeHost,
) -> Result<(), HostError> {
    if !request
        .remote_addr()
        .is_some_and(|address| address.ip().is_loopback())
    {
        return Err(HostError::Boundary);
    }
    let expected = host
        .bind()
        .parse::<SocketAddr>()
        .map_err(|_| HostError::Boundary)?;
    let actual = header(request, "Host")
        .ok_or(HostError::Boundary)?
        .parse::<SocketAddr>()
        .map_err(|_| HostError::Boundary)?;
    if actual != expected {
        return Err(HostError::Boundary);
    }
    Ok(())
}

pub(crate) fn required_origin<'a>(
    request: &'a Request,
    host: &LocalRuntimeHost,
) -> Result<&'a str, HostError> {
    let origin = header(request, "Origin").ok_or(HostError::Boundary)?;
    host.origin_allowed(origin)
        .then_some(origin)
        .ok_or(HostError::Boundary)
}

pub(crate) fn header<'a>(request: &'a Request, name: &'static str) -> Option<&'a str> {
    request
        .headers()
        .iter()
        .find(|header| header.field.equiv(name))
        .map(|header| header.value.as_str())
}

pub(crate) fn bearer(request: &Request) -> Result<&str, HostError> {
    header(request, "Authorization")
        .and_then(|value| value.strip_prefix("Bearer "))
        .filter(|value| !value.is_empty())
        .ok_or(HostError::Unauthorized)
}

pub(crate) fn require_json(request: &Request) -> Result<(), HostError> {
    let content_type = header(request, "Content-Type")
        .and_then(|value| value.split(';').next())
        .map(str::trim);
    (content_type == Some("application/json"))
        .then_some(())
        .ok_or_else(|| HostError::Request("content type must be application/json".to_string()))
}

pub(crate) fn require_client_marker(request: &Request) -> Result<(), HostError> {
    (header(request, "X-NuxtJP-Local-Runtime") == Some("1"))
        .then_some(())
        .ok_or(HostError::Boundary)
}
