use std::collections::BTreeSet;

use sleepy_sdk::{
    validate_desktop_envelope, validate_desktop_request, validate_desktop_result,
    BrightnessSnapshot, CapabilityAvailability, DesktopCapability, DesktopEvent, DesktopResult,
    DesktopSystemUpdate, DesktopUtilitySnapshot, DesktopUtilityUpdate, HyprlandActionCapabilities,
    HyprlandCommand, NightLightSnapshot, ProducerAvailability, RecordingState, RecordingStatus,
    StableId, DESKTOP_WIRE_VERSION,
};

const SNAPSHOT: &str = include_str!("../fixtures/desktop-runtime/full-snapshot.json");
const COMMAND: &str = include_str!("../fixtures/desktop-runtime/command.json");

fn snapshot_value() -> serde_json::Value {
    serde_json::from_str(SNAPSHOT).expect("snapshot fixture must be JSON")
}

fn domain_update(topic: &str, update: serde_json::Value, event_suffix: u8) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": 3,
        "generation": 8,
        "eventId": format!("018f3f4c-8af1-7f6b-bf42-1bd472868e{event_suffix:02x}"),
        "emittedAt": "2026-08-30T12:00:01Z",
        "cause": { "kind": "external" },
        "payload": {
            "type": "domainUpdate",
            "data": { "topic": topic, "update": update }
        }
    })
}

fn available<T>(data: T) -> DesktopCapability<T> {
    DesktopCapability {
        status: CapabilityAvailability::Available,
        data: Some(data),
        diagnostic: None,
    }
}

#[test]
fn explicit_null_never_aliases_absent_v3_terminal_fields() {
    for (name, document) in [
        (
            "capability data",
            serde_json::json!({
                "status": "unsupported",
                "data": null,
                "diagnostic": { "message": "not supported" }
            }),
        ),
        (
            "capability diagnostic",
            serde_json::json!({
                "status": "available",
                "data": { "level": 0.5 },
                "diagnostic": null
            }),
        ),
    ] {
        assert!(
            serde_json::from_value::<DesktopCapability<BrightnessSnapshot>>(document).is_err(),
            "explicit null {name} must fail during deserialization"
        );
    }

    assert!(
        serde_json::from_value::<ProducerAvailability>(
            serde_json::json!({ "status": "available", "diagnostic": null })
        )
        .is_err(),
        "explicit null producer diagnostic must fail during deserialization"
    );
    assert!(
        serde_json::from_value::<DesktopResult>(serde_json::json!({
            "schemaVersion": 3,
            "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e66",
            "generation": 8,
            "status": "succeeded",
            "diagnostic": null
        }))
        .is_err(),
        "explicit null result diagnostic must fail during deserialization"
    );

    for (name, document, round_trip) in [
        (
            "available capability without diagnostic",
            serde_json::json!({ "status": "available", "data": { "level": 0.5 } }),
            serde_json::from_value::<DesktopCapability<BrightnessSnapshot>>,
        ),
        (
            "unavailable capability without data",
            serde_json::json!({
                "status": "unsupported",
                "diagnostic": { "message": "not supported" }
            }),
            serde_json::from_value::<DesktopCapability<BrightnessSnapshot>>,
        ),
    ] {
        let parsed = round_trip(document.clone()).unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(serde_json::to_value(parsed).unwrap(), document, "{name}");
    }

    let producer = serde_json::json!({ "status": "available" });
    assert_eq!(
        serde_json::to_value(
            serde_json::from_value::<ProducerAvailability>(producer.clone()).unwrap()
        )
        .unwrap(),
        producer
    );
    let result = serde_json::json!({
        "schemaVersion": 3,
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e66",
        "generation": 8,
        "status": "succeeded"
    });
    assert_eq!(
        serde_json::to_value(serde_json::from_value::<DesktopResult>(result.clone()).unwrap())
            .unwrap(),
        result
    );
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
fn display_subproducers_degrade_independently_and_reject_aggregate_aliases() {
    for (failed, sibling) in [("brightness", "nightLight"), ("nightLight", "brightness")] {
        let mut candidate = snapshot_value();
        candidate["payload"]["data"]["system"][failed] = serde_json::json!({
            "status": "unsupported",
            "diagnostic": { "message": format!("{failed} is not supported") }
        });
        assert!(
            validate_desktop_envelope(&candidate.to_string()).is_ok(),
            "{failed} failure must not hide available {sibling} state"
        );
        assert_eq!(
            candidate["payload"]["data"]["system"][sibling]["status"],
            "available"
        );
    }

    for required in ["brightness", "nightLight"] {
        let mut missing = snapshot_value();
        missing["payload"]["data"]["system"]
            .as_object_mut()
            .unwrap()
            .remove(required);
        assert!(
            validate_desktop_envelope(&missing.to_string()).is_err(),
            "missing {required} capability must fail closed"
        );
    }

    let mut aggregate_alias = snapshot_value();
    aggregate_alias["payload"]["data"]["system"]["display"] = serde_json::json!({
        "status": "available",
        "data": { "brightness": 0.65, "nightLightEnabled": false }
    });
    assert!(
        validate_desktop_envelope(&aggregate_alias.to_string()).is_err(),
        "the flawed aggregate display alias must not survive in v3"
    );
}

#[test]
fn every_utility_subproducer_has_independent_terminal_availability() {
    let stateful = [
        "trayItems",
        "clipboardEntries",
        "recording",
        "idleInhibited",
        "gameMode",
    ];
    let stateless = ["screenshot", "colorPicker"];

    for failed in stateful.into_iter().chain(stateless) {
        let mut candidate = snapshot_value();
        candidate["payload"]["data"]["utilities"][failed] = serde_json::json!({
            "status": "unsupported",
            "diagnostic": { "message": format!("{failed} is not supported") }
        });
        assert!(
            validate_desktop_envelope(&candidate.to_string()).is_ok(),
            "{failed} must terminate independently without hiding its siblings"
        );
        assert_eq!(
            candidate["payload"]["data"]["utilities"]["recording"]["status"],
            if failed == "recording" {
                "unsupported"
            } else {
                "available"
            }
        );
    }

    for required in stateful.into_iter().chain(stateless) {
        let mut missing = snapshot_value();
        missing["payload"]["data"]["utilities"]
            .as_object_mut()
            .unwrap()
            .remove(required);
        assert!(
            validate_desktop_envelope(&missing.to_string()).is_err(),
            "missing {required} terminal state must fail closed"
        );
    }

    let mut aggregate_alias = snapshot_value();
    aggregate_alias["payload"]["data"]["utilities"]["availability"] =
        serde_json::json!({ "status": "available" });
    assert!(
        validate_desktop_envelope(&aggregate_alias.to_string()).is_err(),
        "aggregate utility availability must not survive in v3"
    );
}

#[test]
fn exact_display_and_utility_subproducer_updates_are_closed() {
    let snapshot = snapshot_value();
    let data = &snapshot["payload"]["data"];
    let updates = [
        (
            "system",
            serde_json::json!({ "domain": "brightness", "data": data["system"]["brightness"] }),
        ),
        (
            "system",
            serde_json::json!({ "domain": "nightLight", "data": data["system"]["nightLight"] }),
        ),
        (
            "utilities",
            serde_json::json!({ "domain": "trayItems", "data": data["utilities"]["trayItems"] }),
        ),
        (
            "utilities",
            serde_json::json!({ "domain": "clipboardEntries", "data": data["utilities"]["clipboardEntries"] }),
        ),
        (
            "utilities",
            serde_json::json!({ "domain": "recording", "data": data["utilities"]["recording"] }),
        ),
        (
            "utilities",
            serde_json::json!({ "domain": "idleInhibited", "data": data["utilities"]["idleInhibited"] }),
        ),
        (
            "utilities",
            serde_json::json!({ "domain": "gameMode", "data": data["utilities"]["gameMode"] }),
        ),
        (
            "utilities",
            serde_json::json!({ "domain": "screenshot", "data": data["utilities"]["screenshot"] }),
        ),
        (
            "utilities",
            serde_json::json!({ "domain": "colorPicker", "data": data["utilities"]["colorPicker"] }),
        ),
    ];

    for (index, (topic, update)) in updates.into_iter().enumerate() {
        let event = domain_update(topic, update, 0x80 + index as u8);
        assert!(
            validate_desktop_envelope(&event.to_string()).is_ok(),
            "{topic} subproducer update {index} must validate"
        );
    }

    for invalid in [
        serde_json::json!({ "domain": "utilities", "data": data["utilities"] }),
        serde_json::json!({ "domain": "unknownUtility", "data": { "status": "available" } }),
    ] {
        let event = domain_update("utilities", invalid, 0x90);
        assert!(
            validate_desktop_envelope(&event.to_string()).is_err(),
            "utility updates must identify one known exact subproducer"
        );
    }
}

#[test]
fn hyprland_action_capabilities_cover_every_closed_command_exactly() {
    let snapshot = snapshot_value();
    let capabilities = snapshot
        .pointer("/payload/data/compositor/hyprland/data/actionCapabilities")
        .unwrap()
        .as_object()
        .unwrap();
    let commands = [
        (
            "focusWindow",
            serde_json::json!({ "type": "focusWindow", "data": { "windowId": "0x1" } }),
        ),
        (
            "moveWindowToWorkspace",
            serde_json::json!({ "type": "moveWindowToWorkspace", "data": { "windowId": "0x1", "workspaceId": "2" } }),
        ),
        (
            "closeWindow",
            serde_json::json!({ "type": "closeWindow", "data": { "windowId": "0x1" } }),
        ),
        (
            "focusWorkspace",
            serde_json::json!({ "type": "focusWorkspace", "data": { "workspaceId": "2" } }),
        ),
        (
            "moveWorkspaceToMonitor",
            serde_json::json!({ "type": "moveWorkspaceToMonitor", "data": { "workspaceId": "2", "monitorId": "DP-1" } }),
        ),
        (
            "toggleFullscreen",
            serde_json::json!({ "type": "toggleFullscreen", "data": { "windowId": "0x1" } }),
        ),
        (
            "toggleFloating",
            serde_json::json!({ "type": "toggleFloating", "data": { "windowId": "0x1" } }),
        ),
        (
            "togglePinned",
            serde_json::json!({ "type": "togglePinned", "data": { "windowId": "0x1" } }),
        ),
        (
            "toggleGroup",
            serde_json::json!({ "type": "toggleGroup", "data": { "windowId": "0x1" } }),
        ),
        ("exit", serde_json::json!({ "type": "exit" })),
    ];

    assert_eq!(
        capabilities
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        commands
            .iter()
            .map(|(name, _)| *name)
            .collect::<BTreeSet<_>>()
    );
    for (name, command) in commands {
        assert!(capabilities[name].is_boolean(), "{name} must be explicit");
        let request = serde_json::json!({
            "schemaVersion": 3,
            "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e66",
            "expectedGeneration": 7,
            "command": { "family": "compositor", "command": command }
        });
        assert!(validate_desktop_request(&request.to_string()).is_ok());

        let mut missing = snapshot_value();
        missing
            .pointer_mut("/payload/data/compositor/hyprland/data/actionCapabilities")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(name);
        assert!(
            validate_desktop_envelope(&missing.to_string()).is_err(),
            "missing {name} capability must fail closed"
        );
    }

    let mut unknown = snapshot_value();
    unknown["payload"]["data"]["compositor"]["hyprland"]["data"]["actionCapabilities"]
        ["unknownAction"] = serde_json::json!(false);
    assert!(validate_desktop_envelope(&unknown.to_string()).is_err());
}

#[test]
fn corrected_v3_capability_types_are_public_and_serialize_to_exact_update_domains() {
    let utility_snapshot = DesktopUtilitySnapshot {
        tray_items: available(Vec::new()),
        clipboard_entries: available(Vec::new()),
        recording: available(RecordingState {
            status: RecordingStatus::Inactive,
            recording_id: None,
            output_id: None,
        }),
        idle_inhibited: available(false),
        game_mode: available(false),
        screenshot: ProducerAvailability {
            status: CapabilityAvailability::Available,
            diagnostic: None,
        },
        color_picker: ProducerAvailability {
            status: CapabilityAvailability::Available,
            diagnostic: None,
        },
    };
    let action_capabilities = HyprlandActionCapabilities {
        focus_window: true,
        move_window_to_workspace: true,
        close_window: true,
        focus_workspace: true,
        move_workspace_to_monitor: true,
        toggle_fullscreen: false,
        toggle_floating: true,
        toggle_pinned: true,
        toggle_group: false,
        exit: true,
    };

    assert_eq!(
        serde_json::to_value(available(BrightnessSnapshot { level: 0.65 })).unwrap(),
        serde_json::json!({ "status": "available", "data": { "level": 0.65 } })
    );
    assert_eq!(
        serde_json::to_value(available(NightLightSnapshot { enabled: false })).unwrap(),
        serde_json::json!({ "status": "available", "data": { "enabled": false } })
    );
    assert_eq!(
        serde_json::to_value(&utility_snapshot).unwrap(),
        serde_json::json!({
            "trayItems": { "status": "available", "data": [] },
            "clipboardEntries": { "status": "available", "data": [] },
            "recording": { "status": "available", "data": { "status": "inactive" } },
            "idleInhibited": { "status": "available", "data": false },
            "gameMode": { "status": "available", "data": false },
            "screenshot": { "status": "available" },
            "colorPicker": { "status": "available" }
        })
    );
    assert!(!action_capabilities.toggle_fullscreen);
    assert!(!action_capabilities.toggle_group);
    let stable_id = || StableId("target".into());
    let commands = [
        (
            HyprlandCommand::FocusWindow {
                window_id: stable_id(),
            },
            true,
        ),
        (
            HyprlandCommand::MoveWindowToWorkspace {
                window_id: stable_id(),
                workspace_id: stable_id(),
            },
            true,
        ),
        (
            HyprlandCommand::CloseWindow {
                window_id: stable_id(),
            },
            true,
        ),
        (
            HyprlandCommand::FocusWorkspace {
                workspace_id: stable_id(),
            },
            true,
        ),
        (
            HyprlandCommand::MoveWorkspaceToMonitor {
                workspace_id: stable_id(),
                monitor_id: stable_id(),
            },
            true,
        ),
        (
            HyprlandCommand::ToggleFullscreen {
                window_id: stable_id(),
            },
            false,
        ),
        (
            HyprlandCommand::ToggleFloating {
                window_id: stable_id(),
            },
            true,
        ),
        (
            HyprlandCommand::TogglePinned {
                window_id: stable_id(),
            },
            true,
        ),
        (
            HyprlandCommand::ToggleGroup {
                window_id: stable_id(),
            },
            false,
        ),
        (HyprlandCommand::Exit, true),
    ];
    for (command, expected) in commands {
        assert_eq!(action_capabilities.supports(&command), expected);
    }

    let system_updates = [
        DesktopSystemUpdate::Brightness(available(BrightnessSnapshot { level: 0.65 })),
        DesktopSystemUpdate::NightLight(available(NightLightSnapshot { enabled: false })),
    ];
    assert_eq!(
        system_updates
            .into_iter()
            .map(|update| serde_json::to_value(update).unwrap()["domain"].clone())
            .collect::<Vec<_>>(),
        ["brightness", "nightLight"]
    );

    let updates = [
        DesktopUtilityUpdate::TrayItems(utility_snapshot.tray_items.clone()),
        DesktopUtilityUpdate::ClipboardEntries(utility_snapshot.clipboard_entries.clone()),
        DesktopUtilityUpdate::Recording(utility_snapshot.recording.clone()),
        DesktopUtilityUpdate::IdleInhibited(utility_snapshot.idle_inhibited.clone()),
        DesktopUtilityUpdate::GameMode(utility_snapshot.game_mode.clone()),
        DesktopUtilityUpdate::Screenshot(utility_snapshot.screenshot.clone()),
        DesktopUtilityUpdate::ColorPicker(utility_snapshot.color_picker.clone()),
    ];
    assert_eq!(
        updates
            .into_iter()
            .map(|update| serde_json::to_value(update).unwrap()["domain"].clone())
            .collect::<Vec<_>>(),
        [
            "trayItems",
            "clipboardEntries",
            "recording",
            "idleInhibited",
            "gameMode",
            "screenshot",
            "colorPicker",
        ]
    );
}

#[test]
fn hyprland_group_membership_is_required_and_round_trips_without_address_topology() {
    let ungrouped = snapshot_value();
    let ungrouped_envelope =
        validate_desktop_envelope(&ungrouped.to_string()).expect("grouped false must validate");
    let ungrouped_round_trip =
        serde_json::to_value(ungrouped_envelope).expect("ungrouped snapshot must serialize");
    assert_eq!(
        ungrouped_round_trip.pointer("/payload/data/compositor/hyprland/data/windows/0/grouped"),
        Some(&serde_json::json!(false))
    );

    let mut grouped = snapshot_value();
    grouped["payload"]["data"]["compositor"]["hyprland"]["data"]["windows"][0]["grouped"] =
        serde_json::json!(true);
    let grouped_envelope =
        validate_desktop_envelope(&grouped.to_string()).expect("grouped true must validate");
    assert_eq!(
        serde_json::to_value(grouped_envelope).expect("grouped snapshot must serialize"),
        grouped
    );

    let mut missing = snapshot_value();
    missing["payload"]["data"]["compositor"]["hyprland"]["data"]["windows"][0]
        .as_object_mut()
        .unwrap()
        .remove("grouped");
    assert!(
        validate_desktop_envelope(&missing.to_string()).is_err(),
        "group membership must never default during readback"
    );

    let mut address_topology = snapshot_value();
    address_topology["payload"]["data"]["compositor"]["hyprland"]["data"]["windows"][0]
        ["groupAddresses"] = serde_json::json!(["0x1234", "0x5678"]);
    assert!(
        validate_desktop_envelope(&address_topology.to_string()).is_err(),
        "the v3 UI contract must not expose Hyprland group address topology"
    );
}

#[test]
fn hyprland_window_updates_cannot_omit_group_membership() {
    let snapshot = snapshot_value();
    let mut window =
        snapshot["payload"]["data"]["compositor"]["hyprland"]["data"]["windows"][0].clone();
    window.as_object_mut().unwrap().remove("grouped");
    let update = serde_json::json!({
        "schemaVersion": 3,
        "generation": 8,
        "eventId": "018f3f4c-8af1-7f6b-bf42-1bd472869405",
        "emittedAt": "2026-08-30T12:00:01Z",
        "cause": { "kind": "external" },
        "payload": {
            "type": "domainUpdate",
            "data": {
                "topic": "compositor",
                "update": { "domain": "windows", "data": [window] }
            }
        }
    });

    assert!(
        validate_desktop_envelope(&update.to_string()).is_err(),
        "incremental readback must not default group membership"
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
        ("/payload/data/utilities/trayItems/data", "id"),
        ("/payload/data/utilities/clipboardEntries/data", "id"),
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
    let event = calendar["payload"]["data"]["calendar"]["snapshot"]["events"][0].clone();
    calendar["payload"]["data"]["calendar"]["snapshot"]["events"]
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
        "/payload/data/system/brightness/data/level",
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
        ("/payload/data/utilities/trayItems/data", 1_024),
        ("/payload/data/utilities/clipboardEntries/data", 500),
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
        .pointer_mut("/payload/data/utilities/trayItems/data/0/menu/children")
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

#[test]
fn incremental_updates_cover_every_snapshot_topic_without_resending_a_full_snapshot() {
    let snapshot = snapshot_value();
    let generation = snapshot["generation"].as_u64().unwrap();
    let data = &snapshot["payload"]["data"];
    let updates = [
        (
            "system",
            serde_json::json!({ "domain": "network", "data": data["system"]["network"] }),
        ),
        (
            "compositor",
            serde_json::json!({ "domain": "workspaces", "data": data["compositor"]["hyprland"]["data"]["workspaces"] }),
        ),
        ("notifications", data["notifications"].clone()),
        ("launcher", data["launcher"].clone()),
        ("calendar", data["calendar"].clone()),
        ("weather", data["weather"].clone()),
        ("appearance", data["appearance"].clone()),
        ("resources", data["resources"].clone()),
        (
            "utilities",
            serde_json::json!({ "domain": "recording", "data": data["utilities"]["recording"] }),
        ),
    ];

    for (index, (topic, update)) in updates.into_iter().enumerate() {
        let envelope = serde_json::json!({
            "schemaVersion": 3,
            "generation": generation + 1,
            "eventId": format!("018f3f4c-8af1-7f6b-bf42-1bd472868e{:02x}", 0x70 + index),
            "emittedAt": "2026-08-30T12:00:01Z",
            "cause": { "kind": "external" },
            "payload": {
                "type": "domainUpdate",
                "data": { "topic": topic, "update": update }
            }
        });
        assert!(
            validate_desktop_envelope(&envelope.to_string()).is_ok(),
            "{topic} must have a closed typed incremental update"
        );
    }
}

#[test]
fn v3_commands_cover_required_producer_mutations_without_serialized_secrets() {
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
        serde_json::json!({ "domain": "media", "action": { "type": "transport", "data": { "playerId": "firefox.instance1", "transport": "playPause" } } }),
        serde_json::json!({ "domain": "display", "action": { "type": "setBrightness", "data": { "outputId": "DP-1", "level": 0.5 } } }),
        serde_json::json!({ "domain": "display", "action": { "type": "setNightLightEnabled", "data": { "enabled": true } } }),
        serde_json::json!({ "domain": "power", "action": { "type": "setProfile", "data": { "profile": "balanced" } } }),
    ];

    for (index, action) in actions.into_iter().enumerate() {
        let request = serde_json::json!({
            "schemaVersion": 3,
            "requestId": format!("018f3f4c-8af1-7f6b-bf42-1bd4728690{:02x}", 0x10 + index),
            "expectedGeneration": 7,
            "command": { "family": "system", "command": action }
        });
        assert!(
            validate_desktop_request(&request.to_string()).is_ok(),
            "required v3 system action {index} must validate"
        );
        assert!(!request
            .to_string()
            .to_ascii_lowercase()
            .contains("password"));
    }
}

#[test]
fn tray_menu_ids_are_local_to_an_item_but_unique_within_that_item() {
    let mut separate_items = snapshot_value();
    let mut second = separate_items["payload"]["data"]["utilities"]["trayItems"]["data"][0].clone();
    second["id"] = serde_json::json!("second-tray-item");
    separate_items["payload"]["data"]["utilities"]["trayItems"]["data"]
        .as_array_mut()
        .unwrap()
        .push(second);
    assert!(validate_desktop_envelope(&separate_items.to_string()).is_ok());

    let mut same_item = snapshot_value();
    same_item["payload"]["data"]["utilities"]["trayItems"]["data"][0]["menu"]["children"][0]
        ["id"] =
        same_item["payload"]["data"]["utilities"]["trayItems"]["data"][0]["menu"]["id"].clone();
    assert!(validate_desktop_envelope(&same_item.to_string()).is_err());
}

#[test]
fn snapshot_graph_rejects_orphan_references_and_ambiguous_focus() {
    for (pointer, invalid) in [
        (
            "/payload/data/system/audio/data/streams/0/nodeId",
            serde_json::json!("missing-node"),
        ),
        (
            "/payload/data/compositor/hyprland/data/workspaces/0/monitorId",
            serde_json::json!("missing-monitor"),
        ),
        (
            "/payload/data/compositor/hyprland/data/windows/0/workspaceId",
            serde_json::json!("missing-workspace"),
        ),
    ] {
        let mut candidate = snapshot_value();
        *candidate.pointer_mut(pointer).unwrap() = invalid;
        assert!(
            validate_desktop_envelope(&candidate.to_string()).is_err(),
            "{pointer} must reference a present stable ID"
        );
    }

    for pointer in [
        "/payload/data/compositor/hyprland/data/monitors",
        "/payload/data/compositor/hyprland/data/windows",
    ] {
        let mut candidate = snapshot_value();
        let mut second = candidate.pointer(pointer).unwrap()[0].clone();
        second["id"] = serde_json::json!("second-focused-item");
        candidate
            .pointer_mut(pointer)
            .unwrap()
            .as_array_mut()
            .unwrap()
            .push(second);
        assert!(
            validate_desktop_envelope(&candidate.to_string()).is_err(),
            "{pointer} must contain at most one focused record"
        );
    }
}

#[test]
fn focused_workspace_cardinality_is_enforced_in_full_and_standalone_updates() {
    let mut full_snapshot = snapshot_value();
    let workspaces = full_snapshot
        .pointer_mut("/payload/data/compositor/hyprland/data/workspaces")
        .unwrap()
        .as_array_mut()
        .unwrap();
    let mut second = workspaces[0].clone();
    second["id"] = serde_json::json!("second-focused-workspace");
    workspaces.push(second);
    assert!(
        validate_desktop_envelope(&full_snapshot.to_string()).is_err(),
        "a full snapshot must not expose two focused workspaces"
    );

    let snapshot = snapshot_value();
    let mut update_workspaces = snapshot
        .pointer("/payload/data/compositor/hyprland/data/workspaces")
        .unwrap()
        .as_array()
        .unwrap()
        .clone();
    let mut second = update_workspaces[0].clone();
    second["id"] = serde_json::json!("second-focused-workspace");
    update_workspaces.push(second);
    let standalone_update = serde_json::json!({
        "schemaVersion": 3,
        "generation": 8,
        "eventId": "018f3f4c-8af1-7f6b-bf42-1bd472869401",
        "emittedAt": "2026-08-30T12:00:01Z",
        "cause": { "kind": "external" },
        "payload": {
            "type": "domainUpdate",
            "data": {
                "topic": "compositor",
                "update": { "domain": "workspaces", "data": update_workspaces }
            }
        }
    });
    assert!(
        validate_desktop_envelope(&standalone_update.to_string()).is_err(),
        "a standalone workspace update must enforce its local focus cardinality"
    );
}

#[test]
fn failed_result_diagnostics_are_nonempty_and_timestamps_are_canonical_rfc3339_utc() {
    let empty_diagnostic = serde_json::json!({
        "schemaVersion": 3,
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e66",
        "generation": 8,
        "status": "failed",
        "diagnostic": { "message": " " }
    });
    assert!(validate_desktop_result(&empty_diagnostic.to_string()).is_err());

    for timestamp in [
        "2026-02-31T12:00:00Z",
        "2026-08-30T25:00:00Z",
        "2026-8-30T12:00:00Z",
        "2026-08-30T12:00:00+00:00",
    ] {
        let mut envelope = snapshot_value();
        envelope["emittedAt"] = serde_json::json!(timestamp);
        assert!(
            validate_desktop_envelope(&envelope.to_string()).is_err(),
            "{timestamp} is not canonical UTC RFC3339"
        );
    }
}

#[test]
fn recording_audio_and_deletion_have_typed_bounded_commands() {
    let base = |command| {
        serde_json::json!({
            "schemaVersion": 3,
            "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e66",
            "expectedGeneration": 8,
            "command": { "family": "utility", "command": command }
        })
    };

    assert!(validate_desktop_request(
        &base(serde_json::json!({
            "type": "startRecording",
            "data": { "outputId": "output:DP-1", "target": "region", "audio": true,
                "region": {"x": -100, "y": 20, "width": 640, "height": 480} }
        }))
        .to_string()
    )
    .is_ok());
    assert!(validate_desktop_request(
        &base(serde_json::json!({
            "type": "startRecording",
            "data": { "outputId": "output:DP-1", "target": "window", "audio": false }
        }))
        .to_string()
    )
    .is_err());
    assert!(validate_desktop_request(
        &base(serde_json::json!({
            "type": "deleteRecording",
            "data": { "recordingId": "recording_20260901_12-34-56.mp4" }
        }))
        .to_string()
    )
    .is_ok());
    assert!(validate_desktop_request(
        &base(serde_json::json!({
            "type": "deleteRecording",
            "data": { "recordingId": "../outside.mp4" }
        }))
        .to_string()
    )
    .is_err());
}

#[test]
fn suspend_then_hibernate_is_a_typed_session_command() {
    assert_eq!(
        serde_json::to_value(sleepy_sdk::RecordingTarget::Region).unwrap(),
        "region"
    );
    let request = serde_json::json!({
        "schemaVersion": 3,
        "requestId": "018f3f4c-8af1-7f6b-bf42-1bd472868e66",
        "expectedGeneration": 8,
        "command": { "family": "session", "command": "suspendThenHibernate" }
    });
    assert!(validate_desktop_request(&request.to_string()).is_ok());
}
