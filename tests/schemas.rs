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
fn desktop_v3_schemas_cover_all_topic_updates_and_required_producer_actions() {
    let event_validator = schema("desktop-event-v3.schema.json");
    let snapshot = desktop_fixture("full-snapshot.json");
    let data = &snapshot["payload"]["data"];
    let updates = vec![
        (
            "system",
            serde_json::json!({ "domain": "network", "data": data["system"]["network"] }),
        ),
        (
            "system",
            serde_json::json!({ "domain": "bluetooth", "data": data["system"]["bluetooth"] }),
        ),
        (
            "system",
            serde_json::json!({ "domain": "audio", "data": data["system"]["audio"] }),
        ),
        (
            "system",
            serde_json::json!({ "domain": "media", "data": data["system"]["media"] }),
        ),
        (
            "system",
            serde_json::json!({ "domain": "battery", "data": data["system"]["battery"] }),
        ),
        (
            "system",
            serde_json::json!({ "domain": "display", "data": data["system"]["display"] }),
        ),
        (
            "system",
            serde_json::json!({ "domain": "power", "data": data["system"]["power"] }),
        ),
        (
            "system",
            serde_json::json!({ "domain": "osd", "data": data["system"]["osd"] }),
        ),
        (
            "system",
            serde_json::json!({ "domain": "lock", "data": data["system"]["lock"] }),
        ),
        (
            "compositor",
            serde_json::json!({ "domain": "hyprland", "data": data["compositor"]["hyprland"] }),
        ),
        (
            "compositor",
            serde_json::json!({ "domain": "monitors", "data": data["compositor"]["hyprland"]["data"]["monitors"] }),
        ),
        (
            "compositor",
            serde_json::json!({ "domain": "workspaces", "data": data["compositor"]["hyprland"]["data"]["workspaces"] }),
        ),
        (
            "compositor",
            serde_json::json!({ "domain": "windows", "data": data["compositor"]["hyprland"]["data"]["windows"] }),
        ),
        ("notifications", data["notifications"].clone()),
        ("launcher", data["launcher"].clone()),
        ("calendar", data["calendar"].clone()),
        ("weather", data["weather"].clone()),
        ("appearance", data["appearance"].clone()),
        ("resources", data["resources"].clone()),
        ("utilities", data["utilities"].clone()),
    ];
    for (index, (topic, update)) in updates.into_iter().enumerate() {
        let event = serde_json::json!({
            "schemaVersion": 3,
            "generation": 8,
            "eventId": format!("018f3f4c-8af1-7f6b-bf42-1bd4728691{:02x}", index),
            "emittedAt": "2026-08-30T12:00:01Z",
            "cause": { "kind": "external" },
            "payload": { "type": "domainUpdate", "data": { "topic": topic, "update": update } }
        });
        assert!(
            event_validator.is_valid(&event),
            "event schema must accept {topic} update"
        );
        assert!(sleepy_sdk::validate_desktop_envelope(&event.to_string()).is_ok());
    }

    let command_validator = schema("desktop-command-v3.schema.json");
    let actions = [
        serde_json::json!({ "domain": "network", "action": { "type": "setWifiEnabled", "data": { "enabled": true } } }),
        serde_json::json!({ "domain": "network", "action": { "type": "scanWifi" } }),
        serde_json::json!({ "domain": "network", "action": { "type": "connectWifi", "data": { "accessPointId": "ap-home" } } }),
        serde_json::json!({ "domain": "network", "action": { "type": "disconnect", "data": { "connectionId": "wifi-home" } } }),
        serde_json::json!({ "domain": "bluetooth", "action": { "type": "setPowered", "data": { "powered": true } } }),
        serde_json::json!({ "domain": "bluetooth", "action": { "type": "scan" } }),
        serde_json::json!({ "domain": "bluetooth", "action": { "type": "pair", "data": { "deviceId": "headphones" } } }),
        serde_json::json!({ "domain": "bluetooth", "action": { "type": "connect", "data": { "deviceId": "headphones" } } }),
        serde_json::json!({ "domain": "bluetooth", "action": { "type": "disconnect", "data": { "deviceId": "headphones" } } }),
        serde_json::json!({ "domain": "audio", "action": { "type": "setDefaultNode", "data": { "nodeId": "speaker" } } }),
        serde_json::json!({ "domain": "audio", "action": { "type": "setNodeVolume", "data": { "nodeId": "speaker", "level": 0.5 } } }),
        serde_json::json!({ "domain": "audio", "action": { "type": "setNodeMuted", "data": { "nodeId": "speaker", "muted": true } } }),
        serde_json::json!({ "domain": "audio", "action": { "type": "setStreamVolume", "data": { "streamId": "stream-firefox", "level": 0.5 } } }),
        serde_json::json!({ "domain": "audio", "action": { "type": "setStreamMuted", "data": { "streamId": "stream-firefox", "muted": true } } }),
        serde_json::json!({ "domain": "media", "action": { "type": "transport", "data": { "playerId": "firefox.instance1", "transport": "next" } } }),
        serde_json::json!({ "domain": "display", "action": { "type": "setBrightness", "data": { "outputId": "DP-1", "level": 0.5 } } }),
        serde_json::json!({ "domain": "display", "action": { "type": "setNightLightEnabled", "data": { "enabled": true } } }),
        serde_json::json!({ "domain": "power", "action": { "type": "setProfile", "data": { "profile": "performance" } } }),
    ];
    for (index, action) in actions.into_iter().enumerate() {
        let request = serde_json::json!({
            "schemaVersion": 3,
            "requestId": format!("018f3f4c-8af1-7f6b-bf42-1bd4728692{:02x}", index),
            "expectedGeneration": 7,
            "command": { "family": "system", "command": action }
        });
        assert!(
            command_validator.is_valid(&request),
            "command schema must accept required producer action {index}"
        );
        assert!(sleepy_sdk::validate_desktop_request(&request.to_string()).is_ok());
    }
}

#[test]
fn desktop_v3_shared_corpus_separates_structural_and_semantic_validation() {
    let event_validator = schema("desktop-event-v3.schema.json");
    let command_validator = schema("desktop-command-v3.schema.json");

    let valid_event = desktop_fixture("full-snapshot.json");
    let mut wrong_version = valid_event.clone();
    wrong_version["schemaVersion"] = serde_json::json!(4);
    let mut invalid_level = valid_event.clone();
    invalid_level["payload"]["data"]["system"]["battery"]["data"]["level"] = serde_json::json!(1.1);
    let mut invalid_timestamp = valid_event.clone();
    invalid_timestamp["emittedAt"] = serde_json::json!("2026-02-31T12:00:00Z");
    let mut invalid_nested_timestamp = valid_event.clone();
    invalid_nested_timestamp["payload"]["data"]["calendar"]["snapshot"]["events"][0]["startsAt"] =
        serde_json::json!("2026-02-31T12:00:00Z");
    let mut unknown_nested = valid_event.clone();
    unknown_nested["payload"]["data"]["system"]["battery"]["data"]["password"] =
        serde_json::json!("forbidden");

    for (name, document, accepted) in [
        ("valid full snapshot", valid_event.clone(), true),
        ("wrong version", wrong_version, false),
        ("invalid normalized level", invalid_level, false),
        ("invalid canonical timestamp", invalid_timestamp, false),
        (
            "invalid nested canonical timestamp",
            invalid_nested_timestamp,
            false,
        ),
        ("unknown nested field", unknown_nested, false),
    ] {
        assert_eq!(
            event_validator.is_valid(&document),
            accepted,
            "schema: {name}"
        );
        assert_eq!(
            sleepy_sdk::validate_desktop_envelope(&document.to_string()).is_ok(),
            accepted,
            "Rust: {name}"
        );
    }

    let valid_request = desktop_fixture("command.json");
    let mut traversal = valid_request.clone();
    traversal["command"] = serde_json::json!({ "family": "launcher", "command": {
        "type": "launch", "data": { "schemaVersion": 2, "desktopId": "../App.desktop", "resources": [] }
    }});
    let mut empty_resource = valid_request.clone();
    empty_resource["command"] = serde_json::json!({ "family": "launcher", "command": {
        "type": "launch", "data": { "schemaVersion": 2, "desktopId": "App.desktop", "resources": [""] }
    }});
    let mut slash_desktop_id = valid_request.clone();
    slash_desktop_id["command"] = serde_json::json!({ "family": "launcher", "command": {
        "type": "launch", "data": { "schemaVersion": 2, "desktopId": "org/example/App.desktop", "resources": [] }
    }});
    let mut backslash_desktop_id = valid_request.clone();
    backslash_desktop_id["command"] = serde_json::json!({ "family": "launcher", "command": {
        "type": "launch", "data": { "schemaVersion": 2, "desktopId": "org\\example\\App.desktop", "resources": [] }
    }});
    let mut empty_desktop_id = valid_request.clone();
    empty_desktop_id["command"] = serde_json::json!({ "family": "launcher", "command": {
        "type": "launch", "data": { "schemaVersion": 2, "desktopId": "", "resources": [] }
    }});
    let mut nul_resource = valid_request.clone();
    nul_resource["command"] = serde_json::json!({ "family": "launcher", "command": {
        "type": "launch", "data": { "schemaVersion": 2, "desktopId": "App.desktop", "resources": ["file\0name"] }
    }});
    let mut padded_values = valid_request.clone();
    padded_values["command"] = serde_json::json!({ "family": "launcher", "command": {
        "type": "launch", "data": {
            "schemaVersion": 2,
            "desktopId": " App.desktop",
            "resources": ["  "]
        }
    }});
    let mut oversized_id = valid_request.clone();
    oversized_id["command"] = serde_json::json!({ "family": "compositor", "command": {
        "type": "focusWindow", "data": { "windowId": "x".repeat(257) }
    }});
    for (name, document, accepted) in [
        ("valid request", valid_request, true),
        (
            "padded launcher values allowed by v2 launch contract",
            padded_values,
            true,
        ),
        ("launcher traversal", traversal, false),
        ("launcher desktop ID slash", slash_desktop_id, false),
        ("launcher desktop ID backslash", backslash_desktop_id, false),
        ("empty launcher desktop ID", empty_desktop_id, false),
        ("empty launcher resource", empty_resource, false),
        ("NUL launcher resource", nul_resource, false),
        ("oversized stable ID", oversized_id, false),
    ] {
        assert_eq!(
            command_validator.is_valid(&document),
            accepted,
            "schema: {name}"
        );
        assert_eq!(
            sleepy_sdk::validate_desktop_request(&document.to_string()).is_ok(),
            accepted,
            "Rust: {name}"
        );
    }

    let mut duplicate_key = valid_event.clone();
    let mut duplicate_sample = duplicate_key["payload"]["data"]["resources"]["samples"][0].clone();
    duplicate_sample["cpuUsage"] = serde_json::json!(0.2);
    duplicate_key["payload"]["data"]["resources"]["samples"]
        .as_array_mut()
        .unwrap()
        .push(duplicate_sample);
    let mut orphan = valid_event.clone();
    orphan["payload"]["data"]["system"]["audio"]["data"]["streams"][0]["nodeId"] =
        serde_json::json!("missing");
    let mut ambiguous_focus = valid_event.clone();
    let mut focused =
        ambiguous_focus["payload"]["data"]["compositor"]["hyprland"]["data"]["monitors"][0].clone();
    focused["id"] = serde_json::json!("HDMI-A-1");
    ambiguous_focus["payload"]["data"]["compositor"]["hyprland"]["data"]["monitors"]
        .as_array_mut()
        .unwrap()
        .push(focused);
    let mut reversed_interval = valid_event.clone();
    reversed_interval["payload"]["data"]["calendar"]["snapshot"]["events"][0]["endsAt"] =
        serde_json::json!("2026-08-30T13:00:00Z");
    let mut bad_contrast = valid_event;
    bad_contrast["payload"]["data"]["appearance"]["theme"]["colors"]["textPrimary"] =
        serde_json::json!("#101018");

    for (name, document) in [
        ("duplicate stable key", duplicate_key),
        ("orphan foreign key", orphan),
        ("ambiguous focus", ambiguous_focus),
        ("reversed calendar interval", reversed_interval),
        ("insufficient theme contrast", bad_contrast),
    ] {
        assert!(
            event_validator.is_valid(&document),
            "{name} is structurally valid and belongs to semantic validation"
        );
        assert!(
            sleepy_sdk::validate_desktop_envelope(&document.to_string()).is_err(),
            "Rust semantic validation must reject {name}"
        );
    }

    let mismatched_result = serde_json::json!({
        "schemaVersion": 3,
        "generation": 8,
        "eventId": "018f3f4c-8af1-7f6b-bf42-1bd472869399",
        "emittedAt": "2026-08-30T12:00:01Z",
        "cause": {
            "kind": "request",
            "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472869398"
        },
        "payload": {
            "type": "commandResult",
            "data": {
                "schemaVersion": 3,
                "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472869398",
                "generation": 7,
                "status": "succeeded"
            }
        }
    });
    assert!(
        event_validator.is_valid(&mismatched_result),
        "cross-field result generation correlation is semantic-only"
    );
    assert!(
        sleepy_sdk::validate_desktop_envelope(&mismatched_result.to_string()).is_err(),
        "Rust must reject a result generation that differs from its envelope"
    );

    let mut empty_failed_diagnostic = mismatched_result.clone();
    empty_failed_diagnostic["payload"]["data"]["generation"] = serde_json::json!(8);
    empty_failed_diagnostic["payload"]["data"]["status"] = serde_json::json!("failed");
    empty_failed_diagnostic["payload"]["data"]["diagnostic"] = serde_json::json!({ "message": "" });
    assert!(!event_validator.is_valid(&empty_failed_diagnostic));
    assert!(
        sleepy_sdk::validate_desktop_envelope(&empty_failed_diagnostic.to_string()).is_err(),
        "schema and Rust must both reject an empty failed-result diagnostic"
    );

    let menu_children: Vec<_> = (0..32_768)
        .map(|index| {
            serde_json::json!({
                "id": format!("node-{index}"),
                "label": "item",
                "enabled": true,
                "children": []
            })
        })
        .collect();
    let mut aggregate_menu_overflow = desktop_fixture("full-snapshot.json");
    aggregate_menu_overflow["payload"]["data"]["utilities"]["trayItems"] = serde_json::json!([
        {
            "id": "tray-a",
            "title": "A",
            "menu": { "id": "root", "label": "A", "enabled": true, "children": menu_children }
        },
        {
            "id": "tray-b",
            "title": "B",
            "menu": { "id": "root", "label": "B", "enabled": true, "children": menu_children }
        }
    ]);
    assert!(
        event_validator.is_valid(&aggregate_menu_overflow),
        "recursive tray menu aggregate count is semantic-only"
    );
    assert!(
        sleepy_sdk::validate_desktop_envelope(&aggregate_menu_overflow.to_string()).is_err(),
        "Rust must reject more than 65536 aggregate tray menu nodes"
    );
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
