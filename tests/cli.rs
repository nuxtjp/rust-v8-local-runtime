mod support;

use serde_json::Value;
use std::process::Command;

const CONFIG: &str = "examples/local-simulation.json";

#[test]
fn validate_config_returns_a_closed_non_executing_result() {
    let output = Command::new(support::package_binary("nuxtjp-local-runtime-host"))
        .args(["validate-config", CONFIG])
        .output()
        .expect("run validation");
    assert!(output.status.success());
    let result: Value = serde_json::from_slice(&output.stdout).expect("one JSON result");
    assert_eq!(
        result["schema"],
        "nuxtjp://local-runtime/config-validation/v1"
    );
    assert_eq!(result["valid"], true);
    assert_eq!(result["external_actions"], false);
    assert_eq!(result.as_object().map(serde_json::Map::len), Some(6));
}
