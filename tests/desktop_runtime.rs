use sleepy_sdk::{
    validate_desktop_envelope, validate_desktop_request, validate_desktop_result, DesktopEvent,
    DESKTOP_WIRE_VERSION,
};

const SNAPSHOT: &str = include_str!("../fixtures/desktop-runtime/full-snapshot.json");
const COMMAND: &str = include_str!("../fixtures/desktop-runtime/command.json");

fn snapshot_value() -> serde_json::Value {
    serde_json::from_str(SNAPSHOT).expect("snapshot fixture must be JSON")
}

#[test]
fn reconnect_fixture_round_trips_without_losing_any_desktop_topic() {
    let envelope = validate_desktop_envelope(SNAPSHOT).expect("snapshot fixture must validate");
    assert_eq!(envelope.schema_version, DESKTOP_WIRE_VERSION);
    assert!(matches!(envelope.payload, DesktopEvent::FullSnapshot(_)));

    let actual = serde_json::to_value(envelope).expect("envelope must serialize");
    assert_eq!(actual, snapshot_value());
    let topics = actual["payload"]["data"]
        .as_object()
        .expect("full snapshot data must be an object");
    assert_eq!(
        topics.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "appearance",
            "calendar",
            "compositor",
            "launcher",
            "notifications",
            "resources",
            "system",
            "utilities",
            "weather",
        ]
    );
}

#[test]
fn command_fixture_is_a_deduplicated_generation_guarded_request() {
    let request = validate_desktop_request(COMMAND).expect("command fixture must validate");
    assert_eq!(request.schema_version, DESKTOP_WIRE_VERSION);
    assert_eq!(request.expected_generation, 7);
    assert_eq!(request.request_id, "018f3f4c-8af1-7f6b-bf42-1bd472868e66");
    assert_eq!(
        serde_json::to_value(request).expect("request must serialize"),
        serde_json::from_str::<serde_json::Value>(COMMAND).expect("fixture must be JSON")
    );
}

#[test]
fn unknown_request_fields_cannot_silently_change_command_meaning() {
    assert!(validate_desktop_request(r#"{"schemaVersion":3,"extra":true}"#).is_err());
}

#[test]
fn zero_generations_cannot_make_reconnects_or_mutations_ambiguous() {
    let mut envelope = snapshot_value();
    envelope["generation"] = serde_json::json!(0);
    assert!(validate_desktop_envelope(&envelope.to_string()).is_err());

    let mut request: serde_json::Value = serde_json::from_str(COMMAND).unwrap();
    request["expectedGeneration"] = serde_json::json!(0);
    assert!(validate_desktop_request(&request.to_string()).is_err());
}

#[test]
fn noncanonical_request_and_event_ids_cannot_bypass_deduplication() {
    let mut envelope = snapshot_value();
    envelope["eventId"] = serde_json::json!("018F3F4C-8AF1-7F6B-BF42-1BD472868E65");
    assert!(validate_desktop_envelope(&envelope.to_string()).is_err());

    let mut request: serde_json::Value = serde_json::from_str(COMMAND).unwrap();
    request["requestId"] = serde_json::json!("018f3f4c8af17f6bbf421bd472868e66");
    assert!(validate_desktop_request(&request.to_string()).is_err());
}

#[test]
fn request_causes_cannot_lose_or_spoof_the_request_that_triggered_them() {
    let mut missing = snapshot_value();
    missing["cause"] = serde_json::json!({ "kind": "request" });
    assert!(validate_desktop_envelope(&missing.to_string()).is_err());

    let mut spoofed = snapshot_value();
    spoofed["cause"] = serde_json::json!({
        "kind": "external",
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e66"
    });
    assert!(validate_desktop_envelope(&spoofed.to_string()).is_err());
}

#[test]
fn empty_and_duplicate_stable_ids_cannot_corrupt_incremental_models() {
    for (pointer, id_field) in [
        ("/payload/data/compositor/hyprland/data/monitors", "id"),
        ("/payload/data/compositor/hyprland/data/workspaces", "id"),
        ("/payload/data/compositor/hyprland/data/windows", "id"),
        ("/payload/data/system/network/data/accessPoints", "id"),
        ("/payload/data/system/network/data/connections", "id"),
        ("/payload/data/system/bluetooth/data/devices", "id"),
        ("/payload/data/system/audio/data/nodes", "id"),
        ("/payload/data/system/audio/data/streams", "id"),
        ("/payload/data/system/media/data/players", "id"),
        ("/payload/data/launcher/entries", "id"),
        ("/payload/data/resources/samples", "id"),
        ("/payload/data/utilities/trayItems", "id"),
        ("/payload/data/utilities/clipboardEntries", "id"),
    ] {
        let mut empty = snapshot_value();
        empty
            .pointer_mut(&format!("{pointer}/0/{id_field}"))
            .expect("fixture pointer must exist")
            .clone_from(&serde_json::json!(""));
        assert!(
            validate_desktop_envelope(&empty.to_string()).is_err(),
            "{pointer} must reject an empty stable ID"
        );

        let mut duplicate = snapshot_value();
        let item = duplicate.pointer(&format!("{pointer}/0")).unwrap().clone();
        duplicate
            .pointer_mut(pointer)
            .expect("fixture pointer must exist")
            .as_array_mut()
            .expect("fixture collection must be an array")
            .push(item);
        assert!(
            validate_desktop_envelope(&duplicate.to_string()).is_err(),
            "{pointer} must reject duplicate stable IDs"
        );
    }
}

#[test]
fn reused_calendar_and_notification_children_keep_unique_stable_ids() {
    let mut calendar = snapshot_value();
    let event = calendar["payload"]["data"]["calendar"]["events"][0].clone();
    calendar["payload"]["data"]["calendar"]["events"]
        .as_array_mut()
        .unwrap()
        .push(event);
    assert!(validate_desktop_envelope(&calendar.to_string()).is_err());

    let mut notification = snapshot_value();
    notification["payload"]["data"]["notifications"]["active"][0]["actions"] = serde_json::json!([
        { "id": "open", "label": "Open", "state": "available" },
        { "id": "open", "label": "Open again", "state": "available" }
    ]);
    assert!(validate_desktop_envelope(&notification.to_string()).is_err());
}

#[test]
fn meters_reject_nonfinite_and_unnormalized_values_before_reaching_qml() {
    for pointer in [
        "/payload/data/system/network/data/accessPoints/0/signalLevel",
        "/payload/data/system/audio/data/nodes/0/volume",
        "/payload/data/system/audio/data/streams/0/volume",
        "/payload/data/system/media/data/players/0/progress",
        "/payload/data/resources/samples/0/cpuUsage",
        "/payload/data/resources/samples/0/memoryUsage",
    ] {
        for invalid in [
            serde_json::json!(-0.01),
            serde_json::json!(1.01),
            serde_json::Value::Null,
        ] {
            let mut candidate = snapshot_value();
            *candidate
                .pointer_mut(pointer)
                .expect("fixture pointer must exist") = invalid;
            assert!(
                validate_desktop_envelope(&candidate.to_string()).is_err(),
                "{pointer} must reject non-finite or unnormalized levels"
            );
        }
    }
}

#[test]
fn every_untrusted_collection_has_its_exact_resource_ceiling() {
    let cases = [
        ("/payload/data/compositor/hyprland/data/monitors", 64usize),
        ("/payload/data/compositor/hyprland/data/workspaces", 1_024),
        ("/payload/data/compositor/hyprland/data/windows", 16_384),
        ("/payload/data/system/network/data/accessPoints", 4_096),
        ("/payload/data/system/bluetooth/data/devices", 1_024),
        ("/payload/data/system/audio/data/nodes", 4_096),
        ("/payload/data/system/audio/data/streams", 16_384),
        ("/payload/data/system/media/data/players", 256),
        ("/payload/data/utilities/trayItems", 1_024),
        ("/payload/data/utilities/clipboardEntries", 500),
        ("/payload/data/notifications/active", 500),
    ];

    for (pointer, maximum) in cases {
        let mut candidate = snapshot_value();
        let template = candidate.pointer(pointer).unwrap()[0].clone();
        let items = candidate
            .pointer_mut(pointer)
            .unwrap()
            .as_array_mut()
            .unwrap();
        items.clear();
        for index in 0..=maximum {
            let mut item = template.clone();
            if pointer.ends_with("/active") {
                item["id"] = serde_json::json!((index + 1) as u64);
            } else {
                item["id"] = serde_json::json!(format!("id-{index}"));
            }
            items.push(item);
        }
        assert!(
            validate_desktop_envelope(&candidate.to_string()).is_err(),
            "{pointer} must reject {} items",
            maximum + 1
        );
    }
}

#[test]
fn tray_menu_tree_has_a_total_node_ceiling_not_only_a_root_ceiling() {
    let mut candidate = snapshot_value();
    let children = candidate
        .pointer_mut("/payload/data/utilities/trayItems/0/menu/children")
        .unwrap()
        .as_array_mut()
        .unwrap();
    children.clear();
    for index in 0..65_536usize {
        children.push(serde_json::json!({
            "id": format!("menu-{index}"),
            "label": "Item",
            "enabled": true,
            "children": []
        }));
    }
    assert!(validate_desktop_envelope(&candidate.to_string()).is_err());
}

#[test]
fn snapshots_and_events_can_never_acquire_password_fields() {
    for pointer in [
        "/payload/data/system/network/data/accessPoints/0/password",
        "/payload/data/system/lock/password",
    ] {
        let mut candidate = snapshot_value();
        let (parent, _) = pointer.rsplit_once('/').unwrap();
        candidate
            .pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("password".into(), serde_json::json!("forbidden"));
        assert!(validate_desktop_envelope(&candidate.to_string()).is_err());
    }
}

#[test]
fn general_desktop_ipc_has_no_programmatic_unlock_command() {
    let mut request: serde_json::Value = serde_json::from_str(COMMAND).unwrap();
    request["command"] = serde_json::json!({
        "family": "session",
        "command": "unlock"
    });
    assert!(validate_desktop_request(&request.to_string()).is_err());
}

#[test]
fn command_results_cannot_confirm_a_different_generation_or_request() {
    let succeeded = serde_json::json!({
        "schemaVersion": 3,
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e66",
        "generation": 8,
        "status": "succeeded"
    });
    assert!(validate_desktop_result(&succeeded.to_string()).is_ok());

    let mut envelope = snapshot_value();
    envelope["generation"] = serde_json::json!(8);
    envelope["cause"] = serde_json::json!({
        "kind": "request",
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e66"
    });
    envelope["payload"] = serde_json::json!({ "type": "commandResult", "data": succeeded });
    assert!(validate_desktop_envelope(&envelope.to_string()).is_ok());

    envelope["payload"]["data"]["generation"] = serde_json::json!(7);
    assert!(validate_desktop_envelope(&envelope.to_string()).is_err());
    envelope["payload"]["data"]["generation"] = serde_json::json!(8);
    envelope["cause"]["requestId"] = serde_json::json!("018f3f4c-8af1-7f6b-bf42-1bd472868e68");
    assert!(validate_desktop_envelope(&envelope.to_string()).is_err());

    let mut incoherent = envelope["payload"]["data"].clone();
    incoherent["status"] = serde_json::json!("failed");
    assert!(validate_desktop_result(&incoherent.to_string()).is_err());
    incoherent["diagnostic"] = serde_json::json!({ "message": "readback failed" });
    assert!(validate_desktop_result(&incoherent.to_string()).is_ok());
}
