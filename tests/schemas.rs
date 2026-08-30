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

fn desktop_fixture(path: &str) -> Value {
    let source = std::fs::read_to_string(format!("fixtures/desktop-runtime/{path}"))
        .expect("desktop fixture should exist");
    serde_json::from_str(&source).expect("desktop fixture should be JSON")
}

#[test]
fn desktop_v3_schemas_accept_the_canonical_fixtures_and_reject_unknown_fields() {
    for (schema_name, fixture_name) in [
        ("desktop-event-v3.schema.json", "full-snapshot.json"),
        ("desktop-command-v3.schema.json", "command.json"),
    ] {
        let validator = schema(schema_name);
        let valid = desktop_fixture(fixture_name);
        assert!(
            validator.is_valid(&valid),
            "{schema_name} must accept {fixture_name}"
        );

        let mut unknown = valid;
        unknown["unknownField"] = serde_json::json!(true);
        assert!(
            !validator.is_valid(&unknown),
            "{schema_name} must reject unknown top-level fields"
        );
    }
}

#[test]
fn desktop_v3_event_schema_matches_runtime_collection_and_level_bounds() {
    let validator = schema("desktop-event-v3.schema.json");
    let mut invalid_level = desktop_fixture("full-snapshot.json");
    invalid_level["payload"]["data"]["system"]["audio"]["data"]["nodes"][0]["volume"] =
        serde_json::json!(1.01);
    assert!(!validator.is_valid(&invalid_level));

    let mut too_many_monitors = desktop_fixture("full-snapshot.json");
    let monitor = too_many_monitors["payload"]["data"]["compositor"]["hyprland"]["data"]
        ["monitors"][0]
        .clone();
    too_many_monitors["payload"]["data"]["compositor"]["hyprland"]["data"]["monitors"] =
        serde_json::Value::Array(vec![monitor; 65]);
    assert!(!validator.is_valid(&too_many_monitors));
}

#[test]
fn desktop_v3_command_schema_has_no_unlock_alternative() {
    let validator = schema("desktop-command-v3.schema.json");
    let mut request = desktop_fixture("command.json");
    request["command"] = serde_json::json!({ "family": "session", "command": "unlock" });
    assert!(!validator.is_valid(&request));
}

#[test]
fn desktop_v3_command_schema_covers_every_closed_rust_command_alternative() {
    let validator = schema("desktop-command-v3.schema.json");
    let commands = vec![
        serde_json::json!({ "family": "system", "command": { "capability": "network.enabled", "value": true } }),
        serde_json::json!({ "family": "system", "command": { "capability": "bluetooth.enabled", "value": true } }),
        serde_json::json!({ "family": "system", "command": { "capability": "audio.volume", "value": 0.5 } }),
        serde_json::json!({ "family": "system", "command": { "capability": "audio.muted", "value": false } }),
        serde_json::json!({ "family": "system", "command": { "capability": "audio.microphoneLevel", "value": 0.5 } }),
        serde_json::json!({ "family": "system", "command": { "capability": "audio.microphoneMuted", "value": false } }),
        serde_json::json!({ "family": "system", "command": { "capability": "audio.outputDevice", "value": "speaker" } }),
        serde_json::json!({ "family": "system", "command": { "capability": "display.brightness", "value": 0.5 } }),
        serde_json::json!({ "family": "system", "command": { "capability": "display.nightLightEnabled", "value": true } }),
        serde_json::json!({ "family": "system", "command": { "capability": "power.profile", "value": "balanced" } }),
        serde_json::json!({ "family": "system", "command": { "capability": "media.transport", "value": "playPause" } }),
        serde_json::json!({ "family": "compositor", "command": { "type": "focusWindow", "data": { "windowId": "0x1" } } }),
        serde_json::json!({ "family": "compositor", "command": { "type": "moveWindowToWorkspace", "data": { "windowId": "0x1", "workspaceId": "2" } } }),
        serde_json::json!({ "family": "compositor", "command": { "type": "closeWindow", "data": { "windowId": "0x1" } } }),
        serde_json::json!({ "family": "compositor", "command": { "type": "focusWorkspace", "data": { "workspaceId": "2" } } }),
        serde_json::json!({ "family": "compositor", "command": { "type": "moveWorkspaceToMonitor", "data": { "workspaceId": "2", "monitorId": "DP-1" } } }),
        serde_json::json!({ "family": "compositor", "command": { "type": "toggleFullscreen", "data": { "windowId": "0x1" } } }),
        serde_json::json!({ "family": "compositor", "command": { "type": "toggleFloating", "data": { "windowId": "0x1" } } }),
        serde_json::json!({ "family": "compositor", "command": { "type": "togglePinned", "data": { "windowId": "0x1" } } }),
        serde_json::json!({ "family": "compositor", "command": { "type": "toggleGroup", "data": { "windowId": "0x1" } } }),
        serde_json::json!({ "family": "compositor", "command": { "type": "exit" } }),
        serde_json::json!({ "family": "notification", "command": { "type": "setDnd", "data": { "enabled": true } } }),
        serde_json::json!({ "family": "notification", "command": { "type": "archive", "data": { "notificationId": 1 } } }),
        serde_json::json!({ "family": "notification", "command": { "type": "invokeAction", "data": { "notificationId": 1, "actionId": "open" } } }),
        serde_json::json!({ "family": "launcher", "command": { "type": "launch", "data": { "schemaVersion": 2, "desktopId": "org.example.App.desktop", "resources": [] } } }),
        serde_json::json!({ "family": "appearance", "command": { "type": "applyTheme", "data": { "themeId": "moon" } } }),
        serde_json::json!({ "family": "appearance", "command": { "type": "setWallpaper", "data": { "wallpaperId": "moon" } } }),
        serde_json::json!({ "family": "appearance", "command": { "type": "setReducedMotion", "data": { "enabled": true } } }),
        serde_json::json!({ "family": "appearance", "command": { "type": "setOpaque", "data": { "enabled": true } } }),
        serde_json::json!({ "family": "utility", "command": { "type": "invokeTrayMenu", "data": { "itemId": "tray", "menuId": "open" } } }),
        serde_json::json!({ "family": "utility", "command": { "type": "pasteClipboard", "data": { "entryId": "clip" } } }),
        serde_json::json!({ "family": "utility", "command": { "type": "clearClipboard" } }),
        serde_json::json!({ "family": "utility", "command": { "type": "setIdleInhibited", "data": { "enabled": true } } }),
        serde_json::json!({ "family": "utility", "command": { "type": "startRecording", "data": { "outputId": "DP-1" } } }),
        serde_json::json!({ "family": "utility", "command": { "type": "pauseRecording" } }),
        serde_json::json!({ "family": "utility", "command": { "type": "stopRecording" } }),
        serde_json::json!({ "family": "utility", "command": { "type": "screenshot", "data": { "outputId": "DP-1" } } }),
        serde_json::json!({ "family": "utility", "command": { "type": "pickColor" } }),
        serde_json::json!({ "family": "utility", "command": { "type": "setGameMode", "data": { "enabled": true } } }),
        serde_json::json!({ "family": "session", "command": "lock" }),
        serde_json::json!({ "family": "session", "command": "suspend" }),
        serde_json::json!({ "family": "session", "command": "logout" }),
        serde_json::json!({ "family": "session", "command": "reboot" }),
        serde_json::json!({ "family": "session", "command": "powerOff" }),
    ];

    for (index, command) in commands.into_iter().enumerate() {
        let request = serde_json::json!({
            "schemaVersion": 3,
            "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e66",
            "expectedGeneration": 7,
            "command": command
        });
        assert!(
            sleepy_sdk::validate_desktop_request(&request.to_string()).is_ok(),
            "Rust must accept closed command alternative {index}: {request}"
        );
        assert!(
            validator.is_valid(&request),
            "schema must accept closed command alternative {index}: {request}"
        );
    }
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
