use std::collections::BTreeMap;

use sleepy_sdk::{
    canonicalize_accelerator, validate_keybindings, validate_keybindings_with_reserved,
    validate_plugin_manifest, validate_preset, validate_settings, validate_system_mutation_result,
    validate_system_snapshot, CapabilityErrorKind, CapabilityState, KeybindingConflictKind,
    SemanticAction, SystemMutationValue,
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

    assert_eq!(conflict.kind, KeybindingConflictKind::Reserved);
    assert_eq!(conflict.accelerator, "Mod+Shift+Escape");
    assert_eq!(
        conflict.actions,
        vec!["recovery.shell".to_string(), "launcher.open".to_string()]
    );
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
    assert_eq!(duplicate.kind, KeybindingConflictKind::Duplicate);
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
    assert_eq!(invalid.kind, KeybindingConflictKind::Invalid);
    assert_eq!(invalid.accelerator, "Mod+T");
    assert_eq!(invalid.actions, vec!["surface.teleport.toggle"]);
}

#[test]
fn system_snapshot_validates_a_complete_document_with_nullable_hardware() {
    let snapshot = validate_system_snapshot(&fixture("system/valid.json"))
        .expect("system snapshot should validate");

    assert_eq!(snapshot.capabilities["network"], CapabilityState::Available);
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
        snapshot.diagnostics["bluetooth"].kind,
        CapabilityErrorKind::Unsupported
    );
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
    candidate["capabilities"]["network"] = serde_json::json!("degraded");

    assert!(validate_system_snapshot(&candidate.to_string()).is_err());
}

#[test]
fn system_snapshot_rejects_an_unknown_capability_diagnostic_kind() {
    let source = fixture("system/valid.json");
    let mut candidate: serde_json::Value =
        serde_json::from_str(&source).expect("fixture should be JSON");
    candidate["diagnostics"]["bluetooth"]["kind"] = serde_json::json!("offline");

    assert!(validate_system_snapshot(&candidate.to_string()).is_err());
}

#[test]
fn system_snapshot_supports_distinct_capability_diagnostic_kinds() {
    let source = fixture("system/valid.json");
    let original: serde_json::Value =
        serde_json::from_str(&source).expect("fixture should be JSON");

    for kind in ["unsupported", "timeout", "parse", "command", "busy"] {
        let mut candidate = original.clone();
        candidate["diagnostics"]["bluetooth"]["kind"] = serde_json::json!(kind);

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

    assert_eq!(result.capability, "audio.volume");
    assert_eq!(result.requested_value, SystemMutationValue::Level(0.72));
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
    invalid_level["requestedValue"] = serde_json::json!(1.01);
    assert!(validate_system_mutation_result(&invalid_level.to_string()).is_err());
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
