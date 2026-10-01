//! Pins worker identity, expiry, and resource ceilings before process creation.

use super::{
    contract::{ENGINE_SCHEMA, SCRIPT_ID, SCRIPT_SHA256, WorkerLimits, WorkerPayload},
    digest,
};
use crate::HostError;
use serde::Deserialize;
use std::{fs, path::Path, time::SystemTime};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessEngineConfig {
    pub schema: String,
    pub worker_path: String,
    pub worker_sha256: String,
    pub expires_unix_seconds: u64,
    pub script_id: String,
    pub script_sha256: String,
    pub limits: WorkerLimits,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessView {
    pub capability_id: String,
    pub view_id: String,
    pub payload: WorkerPayload,
}

impl ProcessEngineConfig {
    pub fn validate(&self) -> Result<(), HostError> {
        self.open_verified_worker().map(|_| ())
    }

    pub(super) fn open_verified_worker(&self) -> Result<fs::File, HostError> {
        self.validate_identity()?;
        let path = Path::new(&self.worker_path);
        let canonical = fs::canonicalize(path).map_err(|_| invalid("worker is unavailable"))?;
        if !path.is_absolute() || canonical != path {
            return Err(invalid("worker path must be absolute and canonical"));
        }
        let before = fs::symlink_metadata(path)?;
        if before.file_type().is_symlink() || !before.is_file() || !executable(&before) {
            return Err(invalid("worker must be a regular non-symlink executable"));
        }
        let mut worker = fs::File::open(path)?;
        if !same_file(&before, &worker.metadata()?) {
            return Err(invalid("worker changed while opening"));
        }
        if digest::reader(&mut worker)? != self.worker_sha256 {
            return Err(invalid("worker SHA-256 does not match"));
        }
        Ok(worker)
    }

    fn validate_identity(&self) -> Result<(), HostError> {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|_| invalid("system clock is invalid"))?
            .as_secs();
        if self.schema != ENGINE_SCHEMA
            || self.script_id != SCRIPT_ID
            || self.script_sha256 != SCRIPT_SHA256
            || self.expires_unix_seconds <= now
            || !lower_digest(&self.worker_sha256)
            || !self.limits.validate()
        {
            return Err(invalid("process engine identity or limits are invalid"));
        }
        Ok(())
    }
}

#[cfg(unix)]
fn executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn executable(_: &fs::Metadata) -> bool {
    false
}

#[cfg(unix)]
fn same_file(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(not(unix))]
fn same_file(_: &fs::Metadata, _: &fs::Metadata) -> bool {
    false
}

fn lower_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn invalid(message: &str) -> HostError {
    HostError::Configuration(message.into())
}
