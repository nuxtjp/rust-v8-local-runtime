mod support;

use serde_json::Value;
use std::process::Command;

#[test]
fn opt_in_actual_worker_renders_one_correlated_view() {
    let Some(config) = std::env::var_os("NUXTJP_V8_READY_CONFIG") else {
        return;
    };
    let output = Command::new(support::package_binary("nuxtjp-local-runtime-host"))
        .args(["render".as_ref(), config.as_os_str()])
        .output()
        .expect("run ready renderer");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let view: Value = serde_json::from_slice(&output.stdout).expect("view envelope");
    assert_eq!(view["schema"], "nuxtjp://local-runtime/view/v1");
    assert_eq!(view["band"], "session");
    assert_eq!(view["external_actions"], false);
    assert!(
        view["payload"]["html"]
            .as_str()
            .is_some_and(|html| html.contains("service-card"))
    );
}
