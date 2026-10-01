use crate::HostError;
use url::Url;

pub(crate) fn normalized_origin(value: &str) -> Result<String, HostError> {
    let url = Url::parse(value).map_err(|_| HostError::Boundary)?;
    let loopback_http = url.scheme() == "http"
        && url
            .host_str()
            .is_some_and(|host| matches!(host, "127.0.0.1" | "::1"));
    if !(url.scheme() == "https" || loopback_http)
        || url.username() != ""
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(HostError::Boundary);
    }
    Ok(url.origin().ascii_serialization())
}
