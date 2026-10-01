//! Admits configuration through one no-follow handle and a bounded read.

use super::HostConfig;
use crate::HostError;
use std::{
    fs::{File, Metadata, OpenOptions},
    io::Read,
    path::Path,
};

const CONFIG_MAX: u64 = 1024 * 1024;

pub fn load_config(path: impl AsRef<Path>) -> Result<HostConfig, HostError> {
    let mut file = open_no_follow(path.as_ref())?;
    let metadata = file
        .metadata()
        .map_err(|_| invalid("configuration file could not be inspected"))?;
    if !opened_file_is_regular(&metadata) {
        return Err(invalid(
            "configuration must be a regular non-symlink file of at most 1 MiB",
        ));
    }
    let source = bounded_read(&mut file)?;
    let config: HostConfig = serde_json::from_slice(&source)?;
    config.validate()?;
    Ok(config)
}

#[cfg(unix)]
fn open_no_follow(path: &Path) -> Result<File, HostError> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| invalid("configuration file could not be opened safely"))
}

#[cfg(windows)]
fn open_no_follow(path: &Path) -> Result<File, HostError> {
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|_| invalid("configuration file could not be opened safely"))
}

#[cfg(not(any(unix, windows)))]
fn open_no_follow(_: &Path) -> Result<File, HostError> {
    Err(invalid(
        "secure configuration file admission is unsupported on this platform",
    ))
}

fn opened_file_is_regular(metadata: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::FileTypeExt;
        let kind = metadata.file_type();
        return metadata.is_file() && !kind.is_symlink_dir() && !kind.is_symlink_file();
    }
    #[cfg(not(windows))]
    metadata.is_file()
}

fn bounded_read(file: &mut File) -> Result<Vec<u8>, HostError> {
    let mut source = Vec::new();
    file.take(CONFIG_MAX + 1)
        .read_to_end(&mut source)
        .map_err(|_| invalid("configuration file could not be read"))?;
    if source.len() as u64 > CONFIG_MAX {
        return Err(invalid("configuration file exceeds 1 MiB"));
    }
    Ok(source)
}

fn invalid(message: &str) -> HostError {
    HostError::Configuration(message.to_string())
}

#[cfg(test)]
#[path = "load_tests.rs"]
mod tests;
