use sleepy_sdk::{validate_plugin_manifest, validate_preset, validate_settings};

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
