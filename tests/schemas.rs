use serde_json::Value;

fn fixture(path: &str) -> Value {
    let source =
        std::fs::read_to_string(format!("fixtures/v1/{path}")).expect("fixture should exist");
    serde_json::from_str(&source).expect("fixture should be JSON")
}

fn schema(path: &str) -> jsonschema::Validator {
    let source = std::fs::read_to_string(format!("schemas/{path}")).expect("schema should exist");
    let document = serde_json::from_str(&source).expect("schema should be JSON");
    jsonschema::validator_for(&document).expect("schema should compile")
}

#[test]
fn preset_schema_rejects_a_builtin_preset_with_a_noncanonical_identifier() {
    let validator = schema("preset.schema.json");

    assert!(!validator.is_valid(&fixture("preset/invalid-builtin-id.json")));
}

#[test]
fn preset_schema_rejects_a_user_preset_without_a_uuid_identifier() {
    let validator = schema("preset.schema.json");

    assert!(!validator.is_valid(&fixture("preset/invalid-user-id.json")));
}

#[test]
fn plugin_schema_rejects_every_unsafe_entrypoint_fixture() {
    let validator = schema("plugin.schema.json");

    for path in [
        "plugin/invalid-unsafe-entrypoint.json",
        "plugin/invalid-padded-entrypoint.json",
        "plugin/invalid-backslash-entrypoint.json",
        "plugin/invalid-absolute-entrypoint.json",
        "plugin/invalid-non-qml-entrypoint.json",
    ] {
        assert!(
            !validator.is_valid(&fixture(path)),
            "{path} must be rejected"
        );
    }
}
