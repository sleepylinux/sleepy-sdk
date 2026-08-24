use sleepy_sdk::{
    validate_calendar_snapshot, validate_desktop_launch_request, validate_osd_event,
    validate_weather_snapshot,
};

#[test]
fn desktop_launch_request_names_an_indexed_entry_and_never_accepts_exec_text() {
    let valid = serde_json::json!({
        "schemaVersion": 2,
        "desktopId": "org.example.Editor.desktop",
        "actionId": "new-window",
        "resources": ["file:///tmp/example.txt"]
    });
    validate_desktop_launch_request(&valid.to_string()).unwrap();

    let mut hostile = valid;
    hostile["exec"] = serde_json::json!("editor $(touch /tmp/pwned)");
    assert!(validate_desktop_launch_request(&hostile.to_string()).is_err());
}

#[test]
fn osd_event_has_a_closed_kind_output_and_normalized_level() {
    let valid = serde_json::json!({
        "schemaVersion": 2,
        "outputId": "DP-1",
        "kind": "volume",
        "level": 0.42,
        "muted": false,
        "label": "Speakers"
    });
    validate_osd_event(&valid.to_string()).unwrap();

    let mut overflow = valid;
    overflow["level"] = serde_json::json!(1.1);
    assert!(validate_osd_event(&overflow.to_string()).is_err());
}

#[test]
fn calendar_snapshot_is_bounded_and_reports_each_source_error_separately() {
    let valid = serde_json::json!({
        "schemaVersion": 2,
        "providerId": "local-ics",
        "windowStart": "2026-08-24T00:00:00Z",
        "windowEnd": "2026-11-22T00:00:00Z",
        "events": [{
            "id": "meeting@example", "summary": "Meeting",
            "startsAt": "2026-08-25T10:00:00Z", "endsAt": "2026-08-25T11:00:00Z",
            "allDay": false, "sourceId": "work.ics"
        }],
        "sourceErrors": [{ "sourceId": "broken.ics", "message": "invalid DTSTART" }]
    });
    validate_calendar_snapshot(&valid.to_string()).unwrap();

    let mut reversed = valid;
    reversed["events"][0]["endsAt"] = serde_json::json!("2026-08-25T09:00:00Z");
    assert!(validate_calendar_snapshot(&reversed.to_string()).is_err());
}

#[test]
fn weather_snapshot_carries_attribution_cache_and_no_secret() {
    let valid = serde_json::json!({
        "schemaVersion": 2,
        "providerId": "met.no",
        "location": { "displayName": "Prague", "latitude": 50.0755, "longitude": 14.4378 },
        "status": "offline",
        "cache": "stale",
        "attribution": "Weather data: MET Norway",
        "forecast": [{ "at": "2026-08-24T22:00:00Z", "temperatureC": 18.0, "symbol": "cloudy" }],
        "diagnostic": { "message": "network unavailable" }
    });
    validate_weather_snapshot(&valid.to_string()).unwrap();

    let mut secret = valid;
    secret["apiKey"] = serde_json::json!("forbidden");
    assert!(validate_weather_snapshot(&secret.to_string()).is_err());
}
