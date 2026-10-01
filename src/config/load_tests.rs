//! Proves that path replacement after admission cannot replace opened content.

use super::{bounded_read, open_no_follow};

#[cfg(unix)]
#[test]
fn reads_the_admitted_handle_after_the_path_is_replaced() {
    let root = temporary_dir();
    let admitted_path = root.join("config.json");
    let replacement_path = root.join("replacement.json");
    std::fs::write(&admitted_path, b"admitted").expect("write admitted input");
    std::fs::write(&replacement_path, b"replacement").expect("write replacement input");

    let mut admitted = open_no_follow(&admitted_path).expect("open admitted input");
    std::fs::rename(&replacement_path, &admitted_path).expect("replace admitted path");
    assert_eq!(
        bounded_read(&mut admitted).expect("read admitted handle"),
        b"admitted"
    );

    drop(admitted);
    std::fs::remove_dir_all(root).expect("remove temporary directory");
}

#[cfg(unix)]
fn temporary_dir() -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "nuxtjp-runtime-race-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&path).expect("create temporary directory");
    path
}
