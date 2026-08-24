use std::collections::BTreeMap;

use sleepy_sdk::{
    canonicalize_accelerator, packaged_reserved_keybindings, validate_keybindings,
    validate_keybindings_with_reserved, validate_plugin_manifest, validate_preset,
    validate_session_action_request, validate_session_action_result, validate_settings,
    validate_system_mutation_result, validate_system_snapshot, AudioOutputDevice,
    CapabilityErrorKind, CapabilityId, CapabilityState, ConflictKind, MediaTransport, PowerProfile,
    SemanticAction, SessionAction, SessionActionStatus, SystemMutation,
};

fn fixture(path: &str) -> String {
    std::fs::read_to_string(format!("fixtures/v1/{path}")).expect("fixture should exist")
}

#[test]
fn validates_a_complete_settings_document() {
    let document =
        validate_settings(&fixture("settings/valid.json")).expect("settings should validate");

    assert_eq!(document.active_preset_id, "builtin.sleepy");
}

#[test]
fn rejects_an_unsupported_settings_schema_version() {
    let result = validate_settings(&fixture("settings/invalid-schema-version.json"));

    assert!(result.is_err());
}

#[test]
fn rejects_unknown_settings_fields() {
    let result = validate_settings(&fixture("settings/invalid-unknown-field.json"));

    assert!(result.is_err());
}

#[test]
fn validates_the_canonical_builtin_preset() {
    let document =
        validate_preset(&fixture("preset/valid-builtin.json")).expect("preset should validate");

    assert_eq!(document.id, "builtin.sleepy");
}

#[test]
fn validates_a_user_preset_with_a_uuid_identifier() {
    let document =
        validate_preset(&fixture("preset/valid-user.json")).expect("preset should validate");

    assert_eq!(document.id, "1f4c9092-29ed-4a2d-85ca-26acfb9d12b3");
}

#[test]
fn rejects_a_user_preset_without_a_uuid_identifier() {
    let result = validate_preset(&fixture("preset/invalid-user-id.json"));

    assert!(result.is_err());
}

#[test]
fn rejects_a_builtin_preset_other_than_builtin_sleepy() {
    let result = validate_preset(&fixture("preset/invalid-builtin-id.json"));

    assert!(result.is_err());
}

#[test]
fn rejects_unknown_preset_fields() {
    let result = validate_preset(&fixture("preset/invalid-unknown-field.json"));

    assert!(result.is_err());
}

#[test]
fn keybinding_canonicalizes_modifier_order_and_case() {
    let accelerator =
        canonicalize_accelerator("shift+mod+d").expect("accelerator should canonicalize");

    assert_eq!(accelerator, "Mod+Shift+D");
}

#[test]
fn keybinding_canonicalizes_ascii_function_named_and_xf86_keys() {
    for (input, expected) in [
        ("  mod+d  ", "Mod+D"),
        ("ctrl+7", "Ctrl+7"),
        ("alt+f1", "Alt+F1"),
        ("shift+F24", "Shift+F24"),
        ("mod+return", "Mod+Return"),
        ("mod+LEFT", "Mod+Left"),
        ("mod+pageup", "Mod+Page_Up"),
        ("XF86AUDIOPLAY", "XF86AudioPlay"),
        ("mod+xf86audioraisevolume", "Mod+XF86AudioRaiseVolume"),
        ("xf86audiolowervolume", "XF86AudioLowerVolume"),
        ("xf86monbrightnessup", "XF86MonBrightnessUp"),
    ] {
        assert_eq!(
            canonicalize_accelerator(input).expect("accelerator should canonicalize"),
            expected,
            "unexpected canonical form for {input}"
        );
    }
}

#[test]
fn keybinding_rejects_internal_whitespace_and_invalid_key_tokens() {
    for accelerator in ["Mod + D", "Mod+\tD", "Mod+$", "Mod+Audio-Play", "Mod+é"] {
        assert!(
            canonicalize_accelerator(accelerator).is_err(),
            "{accelerator:?} must be rejected"
        );
    }
}

#[test]
fn keybinding_rejects_duplicate_and_unknown_modifiers() {
    for accelerator in ["mod+Mod+D", "Hyper+Shift+D"] {
        assert!(
            canonicalize_accelerator(accelerator).is_err(),
            "{accelerator} must be rejected"
        );
    }
}

#[test]
fn keybinding_requires_exactly_one_key() {
    for accelerator in ["Mod+Shift", "D+F"] {
        assert!(
            canonicalize_accelerator(accelerator).is_err(),
            "{accelerator} must be rejected"
        );
    }
}

#[test]
fn keybinding_rejects_a_blank_action_identifier() {
    let bindings = BTreeMap::from([("  ".to_string(), "Mod+D".to_string())]);

    assert!(validate_keybindings(&bindings).is_err());
}

#[test]
fn keybinding_rejects_duplicate_canonical_chords() {
    let bindings = BTreeMap::from([
        ("launcher.open".to_string(), "shift+mod+d".to_string()),
        (
            "surface.controlCenter.toggle".to_string(),
            "Mod+Shift+D".to_string(),
        ),
    ]);

    assert!(validate_keybindings(&bindings).is_err());
}

#[test]
fn keybinding_validates_a_complete_preset_fixture() {
    let document = validate_preset(&fixture("preset/valid-bindings.json"))
        .expect("binding preset should validate");

    assert_eq!(
        document.keybindings["surface.controlCenter.toggle"],
        "shift+mod+d"
    );
}

#[test]
fn keybinding_rejects_a_preset_fixture_with_a_duplicate_chord() {
    let result = validate_preset(&fixture("preset/invalid-duplicate-binding.json"));

    assert!(result.is_err());
}

#[test]
fn keybinding_registry_accepts_known_and_rejects_unknown_semantic_actions() {
    assert_eq!(
        SemanticAction::try_from("surface.controlCenter.toggle")
            .expect("packaged action should be known"),
        SemanticAction::ControlCenterToggle
    );
    assert!(SemanticAction::try_from("surface.teleport.toggle").is_err());

    let result = validate_preset(&fixture("preset/invalid-unknown-action.json"));
    assert!(result.is_err());
}

#[test]
fn keybinding_registry_distinguishes_session_actions_from_the_power_chooser() {
    for (wire_id, expected) in [
        ("session.reboot", SemanticAction::SessionReboot),
        ("session.powerOff", SemanticAction::SessionPowerOff),
        ("session.power", SemanticAction::SessionPower),
    ] {
        assert_eq!(
            SemanticAction::try_from(wire_id).expect("semantic action should be known"),
            expected
        );
    }
}

#[test]
fn keybinding_reports_a_structured_reserved_chord_collision() {
    let document: serde_json::Value =
        serde_json::from_str(&fixture("preset/invalid-reserved-binding.json"))
            .expect("fixture should be JSON");
    let bindings = document["keybindings"]
        .as_object()
        .expect("fixture should contain keybindings")
        .iter()
        .map(|(action, accelerator)| {
            (
                action.clone(),
                accelerator
                    .as_str()
                    .expect("accelerator should be a string")
                    .to_owned(),
            )
        })
        .collect();
    let reserved = BTreeMap::from([("recovery.shell".to_string(), "shift+mod+escape".to_string())]);

    let conflict = validate_keybindings_with_reserved(&bindings, &reserved)
        .expect_err("reserved chord must conflict");

    assert_eq!(conflict.kind, ConflictKind::Reserved);
    assert_eq!(conflict.accelerator, "Mod+Shift+Escape");
    assert_eq!(
        conflict.actions,
        vec!["recovery.shell".to_string(), "launcher.open".to_string()]
    );
}

#[test]
fn keybinding_normal_validation_enforces_the_exact_packaged_recovery_chord() {
    assert_eq!(
        packaged_reserved_keybindings(),
        BTreeMap::from([("recovery.shell".to_string(), "Mod+Shift+Escape".to_string())])
    );

    let result = validate_preset(&fixture("preset/invalid-reserved-binding.json"));
    assert!(result.is_err());
}

#[test]
fn keybinding_reports_structured_duplicate_and_invalid_conflicts() {
    let duplicate_bindings = BTreeMap::from([
        ("launcher.open".to_string(), "Mod+D".to_string()),
        (
            "surface.controlCenter.toggle".to_string(),
            "mod+d".to_string(),
        ),
    ]);
    let duplicate = validate_keybindings_with_reserved(&duplicate_bindings, &BTreeMap::new())
        .expect_err("duplicate chord must conflict");
    assert_eq!(duplicate.kind, ConflictKind::Duplicate);
    assert_eq!(duplicate.accelerator, "Mod+D");
    assert_eq!(
        duplicate.actions,
        vec![
            "launcher.open".to_string(),
            "surface.controlCenter.toggle".to_string()
        ]
    );

    let invalid_bindings =
        BTreeMap::from([("surface.teleport.toggle".to_string(), "Mod+T".to_string())]);
    let invalid = validate_keybindings_with_reserved(&invalid_bindings, &BTreeMap::new())
        .expect_err("unknown semantic action must conflict");
    assert_eq!(invalid.kind, ConflictKind::Invalid);
    assert_eq!(invalid.accelerator, "Mod+T");
    assert_eq!(invalid.actions, vec!["surface.teleport.toggle"]);
}

#[test]
fn system_snapshot_validates_a_complete_document_with_nullable_hardware() {
    let snapshot = validate_system_snapshot(&fixture("system/valid.json"))
        .expect("system snapshot should validate");

    assert_eq!(
        snapshot.capabilities[&CapabilityId::NetworkEnabled],
        CapabilityState::Available
    );
    assert_eq!(snapshot.generation, 41);
    assert_eq!(snapshot.session_actions.len(), 4);
    assert_eq!(
        snapshot.session_actions[&SessionAction::PowerOff],
        CapabilityState::Available
    );
    assert!(snapshot.bluetooth.is_none());
    assert_eq!(
        snapshot
            .display
            .expect("display state should exist")
            .brightness,
        None
    );
    assert_eq!(
        snapshot
            .power
            .expect("power state should exist")
            .battery_level,
        None
    );
    assert_eq!(
        snapshot.diagnostics[&CapabilityId::BluetoothEnabled].kind,
        CapabilityErrorKind::Unsupported
    );
}

#[test]
fn system_capability_ids_are_a_closed_exact_wire_registry() {
    for wire_id in [
        "network.enabled",
        "bluetooth.enabled",
        "audio.volume",
        "audio.muted",
        "audio.microphoneLevel",
        "audio.microphoneMuted",
        "audio.outputDevice",
        "display.brightness",
        "display.nightLightEnabled",
        "power.profile",
        "battery.status",
        "media.transport",
    ] {
        let capability: CapabilityId = serde_json::from_value(serde_json::json!(wire_id))
            .expect("known capability should deserialize");
        assert_eq!(serde_json::to_value(capability).unwrap(), wire_id);
    }

    assert!(
        serde_json::from_value::<CapabilityId>(serde_json::json!("network.signalLevel")).is_err()
    );
}

#[test]
fn session_actions_are_closed_and_separate_from_state_capabilities() {
    for (wire_id, expected) in [
        ("lock", SessionAction::Lock),
        ("logout", SessionAction::Logout),
        ("reboot", SessionAction::Reboot),
        ("powerOff", SessionAction::PowerOff),
    ] {
        let action: SessionAction =
            serde_json::from_value(serde_json::json!(wire_id)).expect("action should deserialize");
        assert_eq!(action, expected);
    }

    assert!(serde_json::from_value::<CapabilityId>(serde_json::json!("session.lock")).is_err());
    assert!(serde_json::from_value::<SessionAction>(serde_json::json!("suspend")).is_err());
}

#[test]
fn system_snapshot_and_mutation_reject_unknown_capability_ids() {
    assert!(validate_system_snapshot(&fixture("system/invalid-unknown-capability.json")).is_err());

    let source = fixture("system/valid-mutation.json");
    let mut mutation: serde_json::Value =
        serde_json::from_str(&source).expect("fixture should be JSON");
    mutation["mutation"]["capability"] = serde_json::json!("audio.balance");
    assert!(validate_system_mutation_result(&mutation.to_string()).is_err());
}

#[test]
fn system_mutation_is_tagged_with_an_exact_value_type_per_capability() {
    for (value, expected) in [
        (
            serde_json::json!({"capability": "network.enabled", "value": true}),
            SystemMutation::NetworkEnabled(true),
        ),
        (
            serde_json::json!({"capability": "audio.volume", "value": 0.5}),
            SystemMutation::AudioVolume(0.5),
        ),
        (
            serde_json::json!({"capability": "audio.outputDevice", "value": "sink.main"}),
            SystemMutation::AudioOutputDevice("sink.main".to_string()),
        ),
        (
            serde_json::json!({"capability": "power.profile", "value": "balanced"}),
            SystemMutation::PowerProfile(PowerProfile::Balanced),
        ),
        (
            serde_json::json!({"capability": "media.transport", "value": "playPause"}),
            SystemMutation::MediaTransport(MediaTransport::PlayPause),
        ),
    ] {
        assert_eq!(
            serde_json::from_value::<SystemMutation>(value)
                .expect("typed mutation should deserialize"),
            expected
        );
    }
}

#[test]
fn system_mutation_rejects_mismatched_values_and_read_only_battery() {
    for value in [
        serde_json::json!({"capability": "network.enabled", "value": 0.5}),
        serde_json::json!({"capability": "audio.volume", "value": true}),
        serde_json::json!({"capability": "audio.outputDevice", "value": "  "}),
        serde_json::json!({"capability": "power.profile", "value": "turbo"}),
        serde_json::json!({"capability": "media.transport", "value": "stop"}),
        serde_json::json!({"capability": "battery.status", "value": true}),
    ] {
        assert!(
            serde_json::from_value::<SystemMutation>(value.clone()).is_err(),
            "mutation must be rejected: {value}"
        );
    }
}

#[test]
fn system_snapshot_rejects_unknown_fields() {
    let result = validate_system_snapshot(&fixture("system/invalid-unknown-field.json"));

    assert!(result.is_err());
}

#[test]
fn system_snapshot_rejects_out_of_range_normalized_levels() {
    let source = fixture("system/valid.json");
    let original: serde_json::Value =
        serde_json::from_str(&source).expect("fixture should be JSON");

    for (pointer, invalid_level) in [
        ("/network/signalLevel", 1.01),
        ("/audio/volume", -0.01),
        ("/audio/microphoneLevel", 1.01),
        ("/display/brightness", -0.01),
        ("/power/batteryLevel", 1.01),
    ] {
        let mut candidate = original.clone();
        *candidate
            .pointer_mut(pointer)
            .expect("fixture pointer should exist") = serde_json::json!(invalid_level);

        assert!(
            validate_system_snapshot(&candidate.to_string()).is_err(),
            "{pointer} must reject {invalid_level}"
        );
    }
}

#[test]
fn system_snapshot_rejects_an_unknown_capability_state() {
    let source = fixture("system/valid.json");
    let mut candidate: serde_json::Value =
        serde_json::from_str(&source).expect("fixture should be JSON");
    candidate["capabilities"]["network.enabled"] = serde_json::json!("degraded");

    assert!(validate_system_snapshot(&candidate.to_string()).is_err());
}

#[test]
fn system_snapshot_rejects_an_unknown_capability_diagnostic_kind() {
    let source = fixture("system/valid.json");
    let mut candidate: serde_json::Value =
        serde_json::from_str(&source).expect("fixture should be JSON");
    candidate["diagnostics"]["bluetooth.enabled"]["kind"] = serde_json::json!("offline");

    assert!(validate_system_snapshot(&candidate.to_string()).is_err());
}

#[test]
fn system_snapshot_supports_distinct_capability_diagnostic_kinds() {
    let source = fixture("system/valid.json");
    let original: serde_json::Value =
        serde_json::from_str(&source).expect("fixture should be JSON");

    for kind in ["unsupported", "timeout", "parse", "command", "busy"] {
        let mut candidate = original.clone();
        candidate["diagnostics"]["bluetooth.enabled"]["kind"] = serde_json::json!(kind);

        assert!(
            validate_system_snapshot(&candidate.to_string()).is_ok(),
            "diagnostic kind {kind} should be supported"
        );
    }
}

#[test]
fn system_mutation_result_validates_confirmed_readback() {
    let result = validate_system_mutation_result(&fixture("system/valid-mutation.json"))
        .expect("system mutation result should validate");

    assert_eq!(result.generation, 42);
    assert_eq!(result.mutation, SystemMutation::AudioVolume(0.72));
    assert_eq!(
        result.snapshot.audio.expect("audio should exist").volume,
        0.72
    );
}

#[test]
fn system_mutation_result_rejects_unknown_fields_and_invalid_levels() {
    let source = fixture("system/valid-mutation.json");
    let original: serde_json::Value =
        serde_json::from_str(&source).expect("fixture should be JSON");

    let mut unknown_field = original.clone();
    unknown_field["optimistic"] = serde_json::json!(true);
    assert!(validate_system_mutation_result(&unknown_field.to_string()).is_err());

    let mut invalid_level = original;
    invalid_level["mutation"]["value"] = serde_json::json!(1.01);
    assert!(validate_system_mutation_result(&invalid_level.to_string()).is_err());
}

#[test]
fn system_mutation_result_requires_matching_result_and_snapshot_generations() {
    let source = fixture("system/valid-mutation.json");
    let mut candidate: serde_json::Value =
        serde_json::from_str(&source).expect("fixture should be JSON");
    candidate["snapshot"]["generation"] = serde_json::json!(41);

    assert!(validate_system_mutation_result(&candidate.to_string()).is_err());
}

#[test]
fn system_documents_require_positive_caller_supplied_generations() {
    let mut snapshot: serde_json::Value =
        serde_json::from_str(&fixture("system/valid.json")).expect("fixture should be JSON");
    snapshot["generation"] = serde_json::json!(0);
    assert!(validate_system_snapshot(&snapshot.to_string()).is_err());

    let mut mutation: serde_json::Value =
        serde_json::from_str(&fixture("system/valid-mutation.json"))
            .expect("fixture should be JSON");
    mutation["generation"] = serde_json::json!(0);
    mutation["snapshot"]["generation"] = serde_json::json!(0);
    assert!(validate_system_mutation_result(&mutation.to_string()).is_err());

    let mut action_result: serde_json::Value =
        serde_json::from_str(&fixture("system/valid-session-result-initiated.json"))
            .expect("fixture should be JSON");
    action_result["generation"] = serde_json::json!(0);
    assert!(validate_session_action_result(&action_result.to_string()).is_err());
}

#[test]
fn system_snapshot_validates_audio_device_identity_and_default_semantics() {
    let snapshot = validate_system_snapshot(&fixture("system/valid.json"))
        .expect("valid audio devices should validate");
    assert_eq!(
        snapshot.audio.unwrap().output_devices[0],
        AudioOutputDevice {
            id: "sink.main".to_string(),
            label: "Sleepy Speakers".to_string(),
            is_default: true,
        }
    );

    let original: serde_json::Value =
        serde_json::from_str(&fixture("system/valid.json")).expect("fixture should be JSON");
    for candidate in [
        {
            let mut value = original.clone();
            value["audio"]["outputDevices"][1]["id"] = serde_json::json!("sink.main");
            value
        },
        {
            let mut value = original.clone();
            value["audio"]["outputDevices"][1]["isDefault"] = serde_json::json!(true);
            value
        },
        {
            let mut value = original.clone();
            value["audio"]["outputDeviceId"] = serde_json::json!("sink.missing");
            value
        },
    ] {
        assert!(validate_system_snapshot(&candidate.to_string()).is_err());
    }
}

#[test]
fn system_snapshot_validates_typed_power_profile_availability() {
    let snapshot = validate_system_snapshot(&fixture("system/valid.json"))
        .expect("valid power profiles should validate");
    assert_eq!(
        snapshot.power.unwrap().current_profile,
        Some(PowerProfile::Balanced)
    );

    let original: serde_json::Value =
        serde_json::from_str(&fixture("system/valid.json")).expect("fixture should be JSON");
    for candidate in [
        {
            let mut value = original.clone();
            value["power"]["currentProfile"] = serde_json::json!("turbo");
            value
        },
        {
            let mut value = original.clone();
            value["power"]["availableProfiles"] = serde_json::json!(["power-saver", "performance"]);
            value
        },
        {
            let mut value = original.clone();
            value["power"]["availableProfiles"] = serde_json::json!(["balanced", "balanced"]);
            value
        },
    ] {
        assert!(validate_system_snapshot(&candidate.to_string()).is_err());
    }
}

#[test]
fn session_action_request_requires_explicit_confirmation() {
    let request = validate_session_action_request(&fixture("system/valid-session-request.json"))
        .expect("confirmed request should validate");
    assert_eq!(request.action, SessionAction::Reboot);
    assert!(request.confirmed);

    let mut candidate: serde_json::Value =
        serde_json::from_str(&fixture("system/valid-session-request.json"))
            .expect("fixture should be JSON");
    candidate["confirmed"] = serde_json::json!(false);
    assert!(validate_session_action_request(&candidate.to_string()).is_err());
}

#[test]
fn session_action_result_enforces_status_diagnostic_invariants() {
    let initiated =
        validate_session_action_result(&fixture("system/valid-session-result-initiated.json"))
            .expect("initiated result should validate");
    assert_eq!(initiated.generation, 43);
    assert_eq!(initiated.status, SessionActionStatus::Initiated);

    let failed =
        validate_session_action_result(&fixture("system/valid-session-result-failed.json"))
            .expect("failed result should validate");
    assert_eq!(failed.status, SessionActionStatus::Failed);
    assert!(failed.diagnostic.is_some());

    let initiated_source = fixture("system/valid-session-result-initiated.json");
    let mut initiated_with_diagnostic: serde_json::Value =
        serde_json::from_str(&initiated_source).expect("fixture should be JSON");
    initiated_with_diagnostic["diagnostic"] =
        serde_json::json!({"kind": "command", "message": "unexpected"});
    assert!(validate_session_action_result(&initiated_with_diagnostic.to_string()).is_err());

    let failed_source = fixture("system/valid-session-result-failed.json");
    let mut failed_without_diagnostic: serde_json::Value =
        serde_json::from_str(&failed_source).expect("fixture should be JSON");
    failed_without_diagnostic["diagnostic"] = serde_json::Value::Null;
    assert!(validate_session_action_result(&failed_without_diagnostic.to_string()).is_err());
}

#[test]
fn validates_a_plugin_manifest_with_a_relative_qml_entrypoint() {
    let manifest =
        validate_plugin_manifest(&fixture("plugin/valid.json")).expect("manifest should validate");

    assert_eq!(manifest.entrypoint, "Main.qml");
}

#[test]
fn rejects_an_unsupported_plugin_api_version() {
    let result = validate_plugin_manifest(&fixture("plugin/invalid-api-version.json"));

    assert!(result.is_err());
}

#[test]
fn rejects_a_plugin_entrypoint_that_escapes_its_package() {
    let result = validate_plugin_manifest(&fixture("plugin/invalid-unsafe-entrypoint.json"));

    assert!(result.is_err());
}

#[test]
fn rejects_a_plugin_entrypoint_with_padding() {
    let result = validate_plugin_manifest(&fixture("plugin/invalid-padded-entrypoint.json"));

    assert!(result.is_err());
}

#[test]
fn rejects_a_backslash_separated_plugin_entrypoint() {
    let result = validate_plugin_manifest(&fixture("plugin/invalid-backslash-entrypoint.json"));

    assert!(result.is_err());
}

#[test]
fn rejects_an_absolute_plugin_entrypoint() {
    let result = validate_plugin_manifest(&fixture("plugin/invalid-absolute-entrypoint.json"));

    assert!(result.is_err());
}

#[test]
fn rejects_a_plugin_entrypoint_without_a_qml_extension() {
    let result = validate_plugin_manifest(&fixture("plugin/invalid-non-qml-entrypoint.json"));

    assert!(result.is_err());
}

#[test]
fn rejects_unknown_plugin_manifest_fields() {
    let result = validate_plugin_manifest(&fixture("plugin/invalid-unknown-field.json"));

    assert!(result.is_err());
}
