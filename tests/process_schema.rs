use serde_json::Value;

const ENGINE_SCHEMA: &str = "schemas/process-engine-v1.schema.json";
const VIEW_SCHEMA: &str = "schemas/process-view-v1.schema.json";

#[test]
fn process_schemas_are_versioned_and_closed() {
    let engine = schema(ENGINE_SCHEMA);
    let view = schema(VIEW_SCHEMA);
    assert_eq!(engine["$id"], "nuxtjp://local-runtime/process-engine/v1");
    assert_eq!(view["$id"], "nuxtjp://local-runtime/process-view/v1");
    assert_eq!(engine["additionalProperties"], false);
    assert_eq!(
        engine["properties"]["limits"]["additionalProperties"],
        false
    );
    assert_eq!(view["additionalProperties"], false);
    assert_eq!(view["properties"]["payload"]["additionalProperties"], false);
}

#[test]
fn engine_schema_pins_worker_protocol_identity_and_limits() {
    let engine = schema(ENGINE_SCHEMA);
    assert_eq!(
        engine["properties"]["script_id"]["const"],
        "nuxtjp-service-card-v1"
    );
    assert_eq!(
        engine["properties"]["script_sha256"]["const"],
        "bd954b068b08f679b2c90c788b907100a184097693bbf24ddd119937d4119b8e"
    );
    assert_eq!(
        engine["properties"]["limits"]["properties"]["timeout_ms"]["maximum"],
        10_000
    );
}

fn schema(relative: &str) -> Value {
    let path = std::path::Path::new(relative);
    serde_json::from_slice(&std::fs::read(path).expect("schema file")).expect("valid JSON")
}
