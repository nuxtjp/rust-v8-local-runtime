//! Exercises the local configuration file admission boundary.

use nuxtjp_local_runtime_host::load_config;
use std::path::{Path, PathBuf};

const LIMIT: usize = 1024 * 1024;
const FIXTURE: &str = "examples/local-simulation.json";

#[test]
fn accepts_a_valid_configuration_at_the_exact_limit() {
    let root = temporary_dir();
    let path = root.join("exact.json");
    write_padded_fixture(&path, LIMIT);
    load_config(&path).expect("exact-limit configuration");
    std::fs::remove_dir_all(root).expect("remove temporary directory");
}

#[test]
fn rejects_limit_plus_one_without_disclosing_content() {
    let root = temporary_dir();
    let path = root.join("oversized.json");
    write_padded_fixture(&path, LIMIT + 1);
    let error = load_config(&path).expect_err("oversized configuration must fail");
    let message = error.to_string();
    assert!(message.contains("exceeds 1 MiB"));
    assert!(!message.contains("runtime_id"));
    std::fs::remove_dir_all(root).expect("remove temporary directory");
}

#[test]
fn rejects_a_directory_without_reading_it() {
    let root = temporary_dir();
    let error = load_config(&root).expect_err("directory must fail");
    assert!(error.to_string().contains("regular non-symlink"));
    std::fs::remove_dir_all(root).expect("remove temporary directory");
}

#[test]
fn parse_errors_do_not_echo_configuration_content() {
    let root = temporary_dir();
    let path = root.join("invalid.json");
    std::fs::write(&path, br#"{"token":"do-not-disclose"}"#).expect("write invalid input");
    let error = load_config(&path).expect_err("invalid configuration must fail");
    assert!(!error.to_string().contains("do-not-disclose"));
    std::fs::remove_dir_all(root).expect("remove temporary directory");
}

#[cfg(unix)]
#[test]
fn rejects_an_initial_symlink() {
    use std::os::unix::fs::symlink;

    let root = temporary_dir();
    let target = root.join("target.json");
    let link = root.join("link.json");
    std::fs::copy(FIXTURE, &target).expect("copy fixture");
    symlink(&target, &link).expect("create symlink");
    assert!(load_config(&link).is_err());
    std::fs::remove_dir_all(root).expect("remove temporary directory");
}

fn write_padded_fixture(path: &Path, size: usize) {
    let mut source = std::fs::read(FIXTURE).expect("read fixture");
    assert!(source.len() <= size);
    source.resize(size, b' ');
    std::fs::write(path, source).expect("write padded fixture");
}

fn temporary_dir() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "nuxtjp-runtime-config-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&path).expect("create temporary directory");
    path
}
