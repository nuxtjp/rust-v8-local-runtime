use crate::HostError;
use serde::de::DeserializeOwned;
use std::io::Read;
use tiny_http::Request;

pub(crate) fn read_json<T: DeserializeOwned>(
    request: &mut Request,
    maximum_bytes: u64,
) -> Result<T, HostError> {
    let mut bytes = Vec::new();
    request
        .as_reader()
        .take(maximum_bytes.saturating_add(1))
        .read_to_end(&mut bytes)?;
    if bytes.is_empty() || bytes.len() as u64 > maximum_bytes {
        return Err(HostError::Request(
            "request body is empty or exceeds the byte limit".to_string(),
        ));
    }
    Ok(serde_json::from_slice(&bytes)?)
}
