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

#[test]
fn preset_schema_accepts_canonicalizable_accelerators() {
    let validator = schema("preset.schema.json");

    assert!(validator.is_valid(&fixture("preset/valid-bindings.json")));
}

#[test]
fn preset_schema_and_rust_agree_on_accelerator_syntax() {
    let validator = schema("preset.schema.json");

    assert!(validator.is_valid(&fixture("preset/valid-accelerator-syntax.json")));
    assert!(sleepy_sdk::validate_preset(
        &std::fs::read_to_string("fixtures/v1/preset/valid-accelerator-syntax.json")
            .expect("fixture should exist")
    )
    .is_ok());

    for path in [
        "preset/invalid-accelerator-whitespace.json",
        "preset/invalid-accelerator-key-token.json",
    ] {
        assert!(
            !validator.is_valid(&fixture(path)),
            "schema must reject {path}"
        );
        assert!(
            sleepy_sdk::validate_preset(
                &std::fs::read_to_string(format!("fixtures/v1/{path}"))
                    .expect("fixture should exist")
            )
            .is_err(),
            "Rust must reject {path}"
        );
    }
}

#[test]
fn preset_schema_leaves_duplicate_modifiers_and_reserved_chords_to_rust() {
    let validator = schema("preset.schema.json");

    for path in [
        "preset/invalid-duplicate-modifier.json",
        "preset/invalid-reserved-binding.json",
    ] {
        assert!(
            validator.is_valid(&fixture(path)),
            "{path} is syntactically valid"
        );
        assert!(
            sleepy_sdk::validate_preset(
                &std::fs::read_to_string(format!("fixtures/v1/{path}"))
                    .expect("fixture should exist")
            )
            .is_err(),
            "semantic Rust validation must reject {path}"
        );
    }
}

#[test]
fn preset_schema_and_rust_reject_modifier_only_chords() {
    let validator = schema("preset.schema.json");

    for path in [
        "preset/invalid-keyless-modifier.json",
        "preset/invalid-keyless-modifier-chain.json",
    ] {
        assert!(
            !validator.is_valid(&fixture(path)),
            "schema must reject {path}"
        );
        assert!(
            sleepy_sdk::validate_preset(
                &std::fs::read_to_string(format!("fixtures/v1/{path}"))
                    .expect("fixture should exist")
            )
            .is_err(),
            "Rust must reject {path}"
        );
    }
}

#[test]
fn system_schema_accepts_nullable_hardware_states() {
    let validator = schema("system.schema.json");

    assert!(validator.is_valid(&fixture("system/valid.json")));
}

#[test]
fn system_schema_rejects_unknown_fields() {
    let validator = schema("system.schema.json");

    assert!(!validator.is_valid(&fixture("system/invalid-unknown-field.json")));
}

#[test]
fn system_schema_rejects_out_of_range_levels() {
    let validator = schema("system.schema.json");
    let original = fixture("system/valid.json");

    for pointer in [
        "/network/signalLevel",
        "/audio/volume",
        "/audio/microphoneLevel",
        "/display/brightness",
        "/power/batteryLevel",
    ] {
        let mut candidate = original.clone();
        *candidate
            .pointer_mut(pointer)
            .expect("fixture pointer should exist") = serde_json::json!(1.01);

        assert!(!validator.is_valid(&candidate), "{pointer} must be bounded");
    }
}

#[test]
fn system_schema_accepts_a_confirmed_mutation_result() {
    let validator = schema("system.schema.json");

    assert!(validator.is_valid(&fixture("system/valid-mutation.json")));
}

#[test]
fn system_schema_accepts_session_action_request_and_results() {
    let validator = schema("system.schema.json");

    for path in [
        "system/valid-session-request.json",
        "system/valid-session-result-initiated.json",
        "system/valid-session-result-failed.json",
    ] {
        assert!(
            validator.is_valid(&fixture(path)),
            "schema must accept {path}"
        );
    }
}

#[test]
fn system_schema_rejects_mismatched_and_read_only_mutations() {
    let validator = schema("system.schema.json");
    let original = fixture("system/valid-mutation.json");

    for mutation in [
        serde_json::json!({"capability": "audio.volume", "value": true}),
        serde_json::json!({"capability": "network.enabled", "value": 0.5}),
        serde_json::json!({"capability": "power.profile", "value": "turbo"}),
        serde_json::json!({"capability": "battery.status", "value": true}),
    ] {
        let mut candidate = original.clone();
        candidate["mutation"] = mutation;
        assert!(!validator.is_valid(&candidate));
    }
}

#[test]
fn system_schema_enforces_session_status_diagnostic_invariants() {
    let validator = schema("system.schema.json");
    let mut initiated_with_diagnostic = fixture("system/valid-session-result-initiated.json");
    initiated_with_diagnostic["diagnostic"] =
        serde_json::json!({"kind": "command", "message": "unexpected"});
    let mut failed_without_diagnostic = fixture("system/valid-session-result-failed.json");
    failed_without_diagnostic["diagnostic"] = serde_json::Value::Null;

    assert!(!validator.is_valid(&initiated_with_diagnostic));
    assert!(!validator.is_valid(&failed_without_diagnostic));
}

#[test]
fn system_schema_rejects_unknown_capability_ids() {
    let validator = schema("system.schema.json");

    assert!(!validator.is_valid(&fixture("system/invalid-unknown-capability.json")));
}
