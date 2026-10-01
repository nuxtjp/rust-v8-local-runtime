//! Runs one verified descriptor with closed environment, bounded pipes, and a hard deadline.

use super::config::ProcessEngineConfig;
use crate::HostError;
use std::{
    io::{Read, Write},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

pub(super) fn invoke(config: &ProcessEngineConfig, request: &[u8]) -> Result<Vec<u8>, HostError> {
    if request.len() as u64 > config.limits.input_bytes {
        return Err(HostError::Unavailable);
    }
    let worker = config.open_verified_worker()?;
    let mut child = spawn(&worker)?;
    let mut stdin = child.stdin.take().ok_or(HostError::Unavailable)?;
    let stdout = child.stdout.take().ok_or(HostError::Unavailable)?;
    let stderr = child.stderr.take().ok_or(HostError::Unavailable)?;
    let output_max = config.limits.output_bytes;
    let input = request.to_vec();
    let stdin_writer =
        std::thread::spawn(move || stdin.write_all(&input).map_err(|_| HostError::Unavailable));
    let stdout_reader = std::thread::spawn(move || drain(stdout, output_max));
    let stderr_reader = std::thread::spawn(move || drain(stderr, 4_096));
    let status = wait_or_kill(&mut child, config.limits.timeout_ms);
    let input_status = stdin_writer.join().map_err(|_| HostError::Unavailable)?;
    let output = stdout_reader.join().map_err(|_| HostError::Unavailable)??;
    stderr_reader.join().map_err(|_| HostError::Unavailable)??;
    status?;
    input_status?;
    if output.len() as u64 > output_max {
        return Err(HostError::Unavailable);
    }
    Ok(output)
}

#[cfg(target_os = "linux")]
fn spawn(worker: &std::fs::File) -> Result<Child, HostError> {
    use std::os::fd::AsRawFd;

    Command::new(format!("/proc/self/fd/{}", worker.as_raw_fd()))
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| HostError::Unavailable)
}

#[cfg(not(target_os = "linux"))]
fn spawn(_: &std::fs::File) -> Result<Child, HostError> {
    Err(HostError::Unavailable)
}

fn wait_or_kill(child: &mut Child, timeout_ms: u64) -> Result<(), HostError> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(_)) => return Err(HostError::Unavailable),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(2)),
            Ok(None) | Err(_) => {
                terminate(child);
                return Err(HostError::Unavailable);
            }
        }
    }
}

fn terminate(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn drain(mut source: impl Read, maximum: u64) -> Result<Vec<u8>, HostError> {
    let mut kept = Vec::new();
    let mut buffer = [0_u8; 8_192];
    loop {
        let count = source.read(&mut buffer)?;
        if count == 0 {
            return Ok(kept);
        }
        if kept.len() as u64 <= maximum {
            let remaining =
                usize::try_from((maximum + 1).saturating_sub(kept.len() as u64)).unwrap_or(0);
            kept.extend_from_slice(&buffer[..count.min(remaining)]);
        }
    }
}
