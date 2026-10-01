//! Computes executable identity without retaining binary contents in memory.

use sha2::{Digest, Sha256};
use std::io::Read;

pub(super) fn reader(source: &mut impl Read) -> Result<String, crate::HostError> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8_192];
    loop {
        let count = source.read(&mut buffer)?;
        if count == 0 {
            return Ok(format!("{:x}", hasher.finalize()));
        }
        hasher.update(&buffer[..count]);
    }
}

pub fn file(path: &std::path::Path) -> Result<String, crate::HostError> {
    reader(&mut std::fs::File::open(path)?)
}
