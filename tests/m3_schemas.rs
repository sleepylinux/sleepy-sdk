use serde_json::Value;

fn schema(name: &str) -> jsonschema::Validator {
    let source = std::fs::read_to_string(format!("schemas/{name}"))
        .unwrap_or_else(|error| panic!("schema {name} must exist: {error}"));
    let document: Value = serde_json::from_str(&source).expect("schema must be JSON");
    let event_document: Value = serde_json::from_str(
        &std::fs::read_to_string("schemas/event.schema.json").expect("event schema must exist"),
    )
    .expect("event schema must be JSON");
    jsonschema::options()
        .with_resource(
            "https://sleepy-linux.org/schemas/v2/event.schema.json",
            jsonschema::Resource::from_contents(event_document)
                .expect("event schema resource must compile"),
        )
        .build(&document)
        .expect("schema must compile")
}

#[test]
fn m3_public_schemas_compile_and_reject_unknown_top_level_fields() {
    let fixtures = [
        (
            "event.schema.json",
            serde_json::json!({
                "schemaVersion": 2,
                "generation": 1,
                "eventId": "018f3f4c-8af1-7f6b-bf42-1bd472868e65",
                "emittedAt": "2026-08-24T21:00:00Z",
                "cause": { "kind": "lifecycle" },
                "payload": { "type": "lifecycle", "data": { "state": "ready" } }
            }),
        ),
        (
            "notification.schema.json",
            serde_json::json!({
                "schemaVersion": 2, "id": 1, "applicationId": "org.example.App",
                "summary": "Hello", "body": "World", "urgency": "normal",
                "createdAt": "2026-08-24T21:00:00Z", "read": false,
                "archived": false, "actions": []
            }),
        ),
        (
            "theme.schema.json",
            serde_json::json!({
                "schemaVersion": 1,
                "id": "018f3f4c-8af1-7f6b-bf42-1bd472868e65", "name": "Moon",
                "origin": "user", "appearance": "dark", "effects": "none",
                "reducedMotion": true, "opaqueFallback": true,
                "colors": { "background": "#101018", "surface": "#202030",
                    "textPrimary": "#ffffff", "textSecondary": "#d0d0df",
                    "accent": "#b9a7ff", "control": "#b9a7ff" }
            }),
        ),
        (
            "hardware.schema.json",
            serde_json::json!({
                "schemaVersion": 1, "generation": 1, "devices": [],
                "fixture": { "formatVersion": 1, "recordedAt": "2026-08-24T21:00:00Z" }
            }),
        ),
        (
            "installation-profile.schema.json",
            serde_json::json!({
                "schemaVersion": 1, "profileId": "laptop",
                "machine": { "hostName": "sleepy", "system": "x86_64-linux", "desktopEnabled": true },
                "firstBoot": { "state": "pending", "generation": 1 },
                "rollback": { "currentGeneration": 1, "availableGenerations": [1] }
            }),
        ),
        (
            "provider.schema.json",
            serde_json::json!({
                "schemaVersion": 2, "providerId": "met.no", "kind": "weather",
                "status": "online", "cache": "fresh"
            }),
        ),
        (
            "mutation.schema.json",
            serde_json::json!({
                "schemaVersion": 2,
                "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e65",
                "expectedGeneration": 1,
                "command": { "type": "setDnd", "data": { "enabled": true } }
            }),
        ),
    ];

    for (name, fixture) in fixtures {
        let validator = schema(name);
        assert!(
            validator.is_valid(&fixture),
            "{name} must accept its valid fixture"
        );

        let mut unknown = fixture;
        unknown["unknownField"] = serde_json::json!(true);
        assert!(
            !validator.is_valid(&unknown),
            "{name} must deny unknown fields"
        );
    }
}

#[test]
fn event_and_mutation_schemas_match_typed_runtime_invariants() {
    let event_validator = schema("event.schema.json");
    let mismatch = serde_json::json!({
        "schemaVersion": 2,
        "generation": 1,
        "eventId": "018f3f4c-8af1-7f6b-bf42-1bd472868e65",
        "emittedAt": "2026-08-24T21:00:00Z",
        "cause": { "kind": "external" },
        "payload": { "type": "capabilityUpdate", "data": {
            "id": "network", "status": "available",
            "value": { "type": "audio", "data": {
                "outputLevel": 0.5, "outputMuted": false,
                "inputLevel": 0.5, "inputMuted": false
            }}
        }}
    });
    assert!(!event_validator.is_valid(&mismatch));

    let mut lifecycle = serde_json::json!({
        "schemaVersion": 2,
        "generation": 1,
        "eventId": "018f3f4c-8af1-7f6b-bf42-1bd472868e65",
        "emittedAt": "2026-08-24T21:00:00Z",
        "cause": { "kind": "lifecycle" },
        "payload": { "type": "lifecycle", "data": { "state": "ready" } }
    });
    lifecycle["eventId"] = serde_json::json!("018F3F4C-8AF1-7F6B-BF42-1BD472868E65");
    assert!(!event_validator.is_valid(&lifecycle));
    lifecycle["eventId"] = serde_json::json!("018f3f4c-8af1-7f6b-bf42-1bd472868e65");
    lifecycle["emittedAt"] = serde_json::json!("2026-08-24T23:00:00+02:00");
    assert!(!event_validator.is_valid(&lifecycle));

    let mutation_validator = schema("mutation.schema.json");
    let set_volume = serde_json::json!({
        "schemaVersion": 2,
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e65",
        "expectedGeneration": 1,
        "command": { "type": "setCapability", "data": { "mutation": {
            "capability": "audio.volume", "value": 0.5
        }}}
    });
    assert!(mutation_validator.is_valid(&set_volume));

    let mut invalid = set_volume;
    invalid["command"]["data"]["mutation"]["value"] = serde_json::json!(1.1);
    assert!(!mutation_validator.is_valid(&invalid));

    let mut blank_theme = serde_json::json!({
        "schemaVersion": 2,
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e65",
        "expectedGeneration": 1,
        "command": { "type": "applyTheme", "data": { "themeId": "  " } }
    });
    assert!(!mutation_validator.is_valid(&blank_theme));
    blank_theme["command"]["data"]["themeId"] = serde_json::json!("moon");
    assert!(mutation_validator.is_valid(&blank_theme));

    let hardware_validator = schema("hardware.schema.json");
    let incoherent_hardware = serde_json::json!({
        "schemaVersion": 1,
        "generation": 1,
        "devices": [{
            "id": { "kind": "cpu", "value": "cpu0" },
            "capability": "brightness",
            "status": "available"
        }],
        "fixture": { "formatVersion": 1, "recordedAt": "2026-08-24T21:00:00Z" }
    });
    assert!(!hardware_validator.is_valid(&incoherent_hardware));
}
