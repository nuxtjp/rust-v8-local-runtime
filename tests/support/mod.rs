use std::ffi::OsStr;
use std::path::PathBuf;

/// Finds a Cargo-built sibling without embedding the checkout's absolute path.
pub fn package_binary(name: &str) -> PathBuf {
    let test_binary = std::env::current_exe().expect("current test binary");
    let deps_dir = test_binary.parent().expect("test binary directory");
    assert_eq!(deps_dir.file_name(), Some(OsStr::new("deps")));
    deps_dir
        .parent()
        .expect("Cargo profile directory")
        .join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
}
