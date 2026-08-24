use sleepy_sdk::{
    validate_event_envelope, validate_mutation_exchange, validate_mutation_request,
    validate_mutation_result, WIRE_SCHEMA_VERSION,
};

fn full_snapshot_event() -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": 2,
        "generation": 4097,
        "eventId": "018f3f4c-8af1-7f6b-bf42-1bd472868e65",
        "emittedAt": "2026-08-24T21:00:00Z",
        "cause": { "kind": "external" },
        "payload": {
            "type": "fullSnapshot",
            "data": {
                "capabilities": [{
                    "id": "network",
                    "status": "available",
                    "value": {
                        "type": "network",
                        "data": {
                            "wifiEnabled": true,
                            "ethernetConnected": false,
                            "connectivity": "full",
                            "activeConnectionId": "home"
                        }
                    }
                },
                { "id": "bluetooth", "status": "unsupported", "diagnostic": { "message": "not present" } },
                { "id": "audio", "status": "unsupported", "diagnostic": { "message": "not present" } },
                { "id": "battery", "status": "unsupported", "diagnostic": { "message": "not present" } },
                { "id": "brightness", "status": "unsupported", "diagnostic": { "message": "not present" } },
                { "id": "powerProfile", "status": "unsupported", "diagnostic": { "message": "not present" } },
                { "id": "media", "status": "unsupported", "diagnostic": { "message": "not present" } },
                { "id": "nightLight", "status": "unsupported", "diagnostic": { "message": "not present" } },
                { "id": "niri", "status": "unsupported", "diagnostic": { "message": "not present" } },
                { "id": "resources", "status": "unsupported", "diagnostic": { "message": "not present" } }],
                "focusedOutputId": "DP-1"
            }
        }
    })
}

#[test]
fn v2_event_envelope_round_trips_a_typed_snapshot() {
    let event = validate_event_envelope(&full_snapshot_event().to_string()).unwrap();

    assert_eq!(WIRE_SCHEMA_VERSION, 2);
    assert_eq!(event.generation, 4097);
    assert_eq!(event.event_id, "018f3f4c-8af1-7f6b-bf42-1bd472868e65");
    assert_eq!(serde_json::to_value(event).unwrap(), full_snapshot_event());
}

#[test]
fn v2_event_envelope_rejects_unknown_fields_versions_and_zero_generation() {
    let mut unknown = full_snapshot_event();
    unknown["surprise"] = serde_json::json!(true);
    assert!(validate_event_envelope(&unknown.to_string()).is_err());

    let mut version = full_snapshot_event();
    version["schemaVersion"] = serde_json::json!(3);
    assert!(validate_event_envelope(&version.to_string()).is_err());

    let mut zero = full_snapshot_event();
    zero["generation"] = serde_json::json!(0);
    assert!(validate_event_envelope(&zero.to_string()).is_err());
}

#[test]
fn request_cause_requires_a_canonical_uuid_and_external_forbids_one() {
    let mut missing = full_snapshot_event();
    missing["cause"] = serde_json::json!({ "kind": "request" });
    assert!(validate_event_envelope(&missing.to_string()).is_err());

    let mut external_with_request = full_snapshot_event();
    external_with_request["cause"] = serde_json::json!({
        "kind": "external",
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e65"
    });
    assert!(validate_event_envelope(&external_with_request.to_string()).is_err());
}

#[test]
fn mutation_result_requires_exact_confirmed_generation_and_request_identity() {
    let request = serde_json::json!({
        "schemaVersion": 2,
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e65",
        "expectedGeneration": 4096,
        "command": {
            "type": "setDnd",
            "data": { "enabled": true }
        }
    });
    validate_mutation_request(&request.to_string()).unwrap();

    let mut confirmed_event = full_snapshot_event();
    confirmed_event["cause"] = serde_json::json!({
        "kind": "request",
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e65"
    });
    let result = serde_json::json!({
        "schemaVersion": 2,
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e65",
        "generation": 4097,
        "status": "confirmed",
        "confirmedEvent": confirmed_event
    });
    validate_mutation_result(&result.to_string()).unwrap();
    validate_mutation_exchange(&request.to_string(), &result.to_string()).unwrap();

    let mut mismatch = result.clone();
    mismatch["confirmedEvent"]["generation"] = serde_json::json!(4098);
    assert!(validate_mutation_result(&mismatch.to_string()).is_err());

    let mut unrelated = result.clone();
    unrelated["confirmedEvent"]["cause"]["requestId"] =
        serde_json::json!("018f3f4c-8af1-7f6b-bf42-1bd472868e66");
    assert!(validate_mutation_result(&unrelated.to_string()).is_err());

    let mut stale = result;
    stale["generation"] = serde_json::json!(4096);
    stale["confirmedEvent"]["generation"] = serde_json::json!(4096);
    assert!(validate_mutation_exchange(&request.to_string(), &stale.to_string()).is_err());
}

#[test]
fn unavailable_capability_has_a_closed_failure_kind_and_no_value() {
    let mut event = full_snapshot_event();
    event["payload"]["data"]["capabilities"][1] = serde_json::json!({
        "id": "bluetooth",
        "status": "permissionDenied",
        "diagnostic": { "message": "access denied" }
    });
    validate_event_envelope(&event.to_string()).unwrap();

    event["payload"]["data"]["capabilities"][1]["status"] = serde_json::json!("temporarilyBroken");
    assert!(validate_event_envelope(&event.to_string()).is_err());
}

#[test]
fn nested_unknown_fields_are_rejected_in_events_and_commands() {
    let mut event = full_snapshot_event();
    event["payload"]["unknown"] = serde_json::json!(true);
    assert!(validate_event_envelope(&event.to_string()).is_err());

    let request = serde_json::json!({
        "schemaVersion": 2,
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e65",
        "expectedGeneration": 1,
        "command": { "type": "setDnd", "data": { "enabled": true, "unknown": 1 } }
    });
    assert!(validate_mutation_request(&request.to_string()).is_err());
}

#[test]
fn full_snapshot_requires_every_unique_capability_and_matching_typed_values() {
    let mut missing = full_snapshot_event();
    missing["payload"]["data"]["capabilities"]
        .as_array_mut()
        .unwrap()
        .pop();
    assert!(validate_event_envelope(&missing.to_string()).is_err());

    let mut duplicate = full_snapshot_event();
    duplicate["payload"]["data"]["capabilities"][1]["id"] = serde_json::json!("network");
    assert!(validate_event_envelope(&duplicate.to_string()).is_err());

    let mut mismatch = full_snapshot_event();
    mismatch["payload"]["data"]["capabilities"][0]["value"] = serde_json::json!({
        "type": "audio",
        "data": { "outputLevel": 0.5, "outputMuted": false, "inputLevel": 0.5, "inputMuted": false }
    });
    assert!(validate_event_envelope(&mismatch.to_string()).is_err());
}

#[test]
fn rust_validation_matches_nested_schema_bounds_and_nonempty_diagnostics() {
    let mut audio = full_snapshot_event();
    audio["payload"]["data"]["capabilities"][2] = serde_json::json!({
        "id": "audio", "status": "available",
        "value": { "type": "audio", "data": {
            "outputLevel": 1.1, "outputMuted": false, "inputLevel": 0.5, "inputMuted": false
        }}
    });
    assert!(validate_event_envelope(&audio.to_string()).is_err());

    let mut empty_diagnostic = full_snapshot_event();
    empty_diagnostic["payload"]["data"]["capabilities"][1]["diagnostic"]["message"] =
        serde_json::json!("");
    assert!(validate_event_envelope(&empty_diagnostic.to_string()).is_err());

    let mut invalid_timestamp = full_snapshot_event();
    invalid_timestamp["emittedAt"] = serde_json::json!("xTyZ");
    assert!(validate_event_envelope(&invalid_timestamp.to_string()).is_err());

    let mut uppercase_uuid = full_snapshot_event();
    uppercase_uuid["eventId"] = serde_json::json!("018F3F4C-8AF1-7F6B-BF42-1BD472868E65");
    assert!(validate_event_envelope(&uppercase_uuid.to_string()).is_err());
}

#[test]
fn mutation_commands_enforce_their_semantic_bounds() {
    let mut base = serde_json::json!({
        "schemaVersion": 2,
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e65",
        "expectedGeneration": 1,
        "command": { "type": "focusWindow", "data": { "windowId": 0 } }
    });
    assert!(validate_mutation_request(&base.to_string()).is_err());

    base["command"]["data"]["windowId"] = serde_json::json!(42);
    validate_mutation_request(&base.to_string()).unwrap();

    let mut blank_theme = base.clone();
    blank_theme["command"] =
        serde_json::json!({ "type": "applyTheme", "data": { "themeId": "  " } });
    assert!(validate_mutation_request(&blank_theme.to_string()).is_err());

    let mut invalid_volume = base;
    invalid_volume["command"] = serde_json::json!({
        "type": "setCapability",
        "data": { "mutation": { "capability": "audio.volume", "value": 1.1 } }
    });
    assert!(validate_mutation_request(&invalid_volume.to_string()).is_err());
}

#[test]
fn incremental_events_and_mutation_failures_enforce_nonempty_identity() {
    for payload in [
        serde_json::json!({
            "type": "notification", "data": { "notificationId": 0, "change": "added" }
        }),
        serde_json::json!({
            "type": "provider", "data": { "providerId": "  ", "online": false }
        }),
        serde_json::json!({
            "type": "theme", "data": { "themeId": "", "applied": false }
        }),
    ] {
        let mut event = full_snapshot_event();
        event["payload"] = payload;
        assert!(validate_event_envelope(&event.to_string()).is_err());
    }

    let failure = serde_json::json!({
        "schemaVersion": 2,
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e65",
        "generation": 2,
        "status": "rejected",
        "error": { "code": "", "message": "  " }
    });
    assert!(validate_mutation_result(&failure.to_string()).is_err());
}
