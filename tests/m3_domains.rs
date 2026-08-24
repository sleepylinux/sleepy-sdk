use sleepy_sdk::{
    validate_hardware_capability_snapshot, validate_installation_profile,
    validate_notification_document, validate_provider_snapshot, validate_theme_document,
};

#[test]
fn notification_document_is_strict_and_actions_expire_explicitly() {
    let valid = serde_json::json!({
        "schemaVersion": 2,
        "id": 42,
        "applicationId": "org.example.Mail",
        "summary": "New message",
        "body": "Literal <b>text</b>",
        "urgency": "normal",
        "createdAt": "2026-08-24T21:00:00Z",
        "timeoutMs": 5000,
        "read": false,
        "archived": false,
        "actions": [{ "id": "reply", "label": "Reply", "state": "expired" }]
    });
    let notification = validate_notification_document(&valid.to_string()).unwrap();
    assert_eq!(notification.body, "Literal <b>text</b>");

    let mut unknown = valid;
    unknown["markup"] = serde_json::json!(true);
    assert!(validate_notification_document(&unknown.to_string()).is_err());
}

#[test]
fn theme_document_rejects_unknown_colors_and_insufficient_contrast() {
    let valid = serde_json::json!({
        "schemaVersion": 1,
        "id": "018f3f4c-8af1-7f6b-bf42-1bd472868e65",
        "name": "Moonlight",
        "origin": "user",
        "appearance": "dark",
        "effects": "reduced",
        "reducedMotion": true,
        "opaqueFallback": true,
        "colors": {
            "background": "#101018",
            "surface": "#202030",
            "textPrimary": "#ffffff",
            "textSecondary": "#d0d0df",
            "accent": "#b9a7ff",
            "control": "#b9a7ff"
        }
    });
    validate_theme_document(&valid.to_string()).unwrap();

    let mut low_contrast = valid;
    low_contrast["colors"]["textPrimary"] = serde_json::json!("#222232");
    assert!(validate_theme_document(&low_contrast.to_string()).is_err());
}

#[test]
fn hardware_snapshot_has_typed_ids_and_closed_failure_states() {
    let valid = serde_json::json!({
        "schemaVersion": 1,
        "generation": 8,
        "devices": [{
            "id": { "kind": "backlight", "value": "intel_backlight" },
            "capability": "brightness",
            "status": "permissionDenied",
            "diagnostic": { "message": "read-only" }
        }],
        "fixture": { "formatVersion": 1, "recordedAt": "2026-08-24T20:00:00Z" }
    });
    validate_hardware_capability_snapshot(&valid.to_string()).unwrap();

    let mut invalid = valid;
    invalid["devices"][0]["status"] = serde_json::json!("missingMaybe");
    assert!(validate_hardware_capability_snapshot(&invalid.to_string()).is_err());
}

#[test]
fn installation_profile_is_declarative_and_rejects_unknown_versions_or_disk_fields() {
    let valid = serde_json::json!({
        "schemaVersion": 1,
        "profileId": "laptop",
        "machine": {
            "hostName": "sleepy",
            "system": "x86_64-linux",
            "desktopEnabled": true
        },
        "firstBoot": { "state": "pending", "generation": 1 },
        "rollback": { "currentGeneration": 1, "availableGenerations": [1] }
    });
    validate_installation_profile(&valid.to_string()).unwrap();

    let mut disk = valid.clone();
    disk["disk"] = serde_json::json!({ "device": "/dev/vda" });
    assert!(validate_installation_profile(&disk.to_string()).is_err());

    let mut future = valid;
    future["schemaVersion"] = serde_json::json!(2);
    assert!(validate_installation_profile(&future.to_string()).is_err());
}

#[test]
fn provider_snapshot_distinguishes_offline_stale_and_errors_without_secrets() {
    let valid = serde_json::json!({
        "schemaVersion": 2,
        "providerId": "met.no",
        "kind": "weather",
        "status": "offline",
        "cache": "stale",
        "diagnostic": { "message": "network unavailable" }
    });
    validate_provider_snapshot(&valid.to_string()).unwrap();

    let mut secret = valid;
    secret["apiKey"] = serde_json::json!("must-not-exist");
    assert!(validate_provider_snapshot(&secret.to_string()).is_err());
}

#[test]
fn durable_rust_validation_matches_schema_identity_uniqueness_and_diagnostics() {
    let zero_notification = serde_json::json!({
        "schemaVersion": 2, "id": 0, "applicationId": "org.example.App",
        "summary": "Hello", "body": "", "urgency": "normal",
        "createdAt": "2026-08-24T21:00:00Z", "read": false,
        "archived": false, "actions": []
    });
    assert!(validate_notification_document(&zero_notification.to_string()).is_err());

    let duplicate_rollback = serde_json::json!({
        "schemaVersion": 1, "profileId": "laptop",
        "machine": { "hostName": "sleepy", "system": "x86_64-linux", "desktopEnabled": true },
        "firstBoot": { "state": "pending", "generation": 1 },
        "rollback": { "currentGeneration": 1, "availableGenerations": [1, 1] }
    });
    assert!(validate_installation_profile(&duplicate_rollback.to_string()).is_err());

    let incoherent_hardware = serde_json::json!({
        "schemaVersion": 1, "generation": 1,
        "devices": [{
            "id": { "kind": "cpu", "value": "cpu0" }, "capability": "brightness",
            "status": "error", "diagnostic": { "message": "" }
        }],
        "fixture": { "formatVersion": 1, "recordedAt": "2026-08-24T21:00:00Z" }
    });
    assert!(validate_hardware_capability_snapshot(&incoherent_hardware.to_string()).is_err());
}
