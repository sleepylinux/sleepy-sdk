use serde_json::{json, Value};
use sleepy_sdk::{validate_capture_reply, validate_capture_request};

const ID: &str = "018f3f4c-8af1-7f6b-bf42-1bd472868e65";

fn request(kind: &str) -> Value {
    let mut command = json!({"type": kind, "jobId": ID});
    if kind == "begin" {
        command["outputId"] = json!("output:DP-1");
    }
    json!({"schemaVersion":1,"command":command})
}

fn reply(state: &str) -> Value {
    let mut job = json!({"jobId":ID,"outputId":"output:DP-1","state":state});
    if state == "completed" {
        job["result"] = json!({"path":format!("/run/user/1000/sleepy/captures/screenshot-{ID}.png"),"mimeType":"image/png","width":1920,"height":1080});
    }
    if state == "failed" {
        job["diagnostic"] = json!({"code":"captureFailed","message":"Capture failed"});
    }
    json!({"schemaVersion":1,"payload":{"type":"job","job":job}})
}

fn accepts_request(value: &Value) -> bool {
    validate_capture_request(&value.to_string()).is_ok()
}
fn accepts_reply(value: &Value) -> bool {
    validate_capture_reply(&value.to_string()).is_ok()
}

#[test]
fn requests_are_bounded_and_closed() {
    for kind in ["begin", "status", "cancel"] {
        assert!(accepts_request(&request(kind)));
    }
    assert!(accepts_request(
        &json!({"schemaVersion":1,"command":{"type":"capabilities"}})
    ));
    for id in ["", "../../escape", "018F3F4C-8AF1-7F6B-BF42-1BD472868E65"] {
        let mut value = request("begin");
        value["command"]["jobId"] = json!(id);
        assert!(!accepts_request(&value));
    }
    for output in ["DP-1", "output:", "output:DP-1;sh", "output:é"] {
        let mut value = request("begin");
        value["command"]["outputId"] = json!(output);
        assert!(!accepts_request(&value));
    }
    let mut value = request("status");
    value["command"]["outputPath"] = json!("/tmp/a");
    assert!(!accepts_request(&value));
    let mut value = request("begin");
    value["schemaVersion"] = json!(3);
    assert!(!accepts_request(&value));
}

#[test]
fn terminal_states_require_exactly_their_result() {
    for state in [
        "awaitingConsent",
        "capturing",
        "completed",
        "cancelled",
        "failed",
    ] {
        assert!(accepts_reply(&reply(state)));
    }
    for state in ["awaitingConsent", "capturing", "cancelled", "failed"] {
        let mut value = reply(state);
        value["payload"]["job"]["result"] = reply("completed")["payload"]["job"]["result"].clone();
        assert!(!accepts_reply(&value));
    }
    for state in ["completed", "failed"] {
        let mut value = reply(state);
        value["payload"]["job"]
            .as_object_mut()
            .unwrap()
            .remove(if state == "completed" {
                "result"
            } else {
                "diagnostic"
            });
        assert!(!accepts_reply(&value));
    }
    let mut value = reply("cancelled");
    value["payload"]["job"]["diagnostic"] = json!({"code":"captureFailed","message":"error"});
    assert!(!accepts_reply(&value));
    let mut value = reply("capturing");
    value["payload"]["job"]["result"] = Value::Null;
    assert!(!accepts_reply(&value));
}

#[test]
fn png_result_cannot_name_an_unrelated_or_traversing_path() {
    for path in [
        "/tmp/screenshot.png",
        "/run/user/1000/sleepy/captures/../secret.png",
        "/run/user/1000/sleepy/captures/screenshot-other.png",
        "/run/user/01000/sleepy/captures/screenshot-018f3f4c-8af1-7f6b-bf42-1bd472868e65.png",
    ] {
        let mut value = reply("completed");
        value["payload"]["job"]["result"]["path"] = json!(path);
        assert!(!accepts_reply(&value));
    }
    for (key, bad) in [
        ("width", json!(0)),
        ("height", json!(32769)),
        ("mimeType", json!("image/jpeg")),
    ] {
        let mut value = reply("completed");
        value["payload"]["job"]["result"][key] = bad;
        assert!(!accepts_reply(&value));
    }
}

#[test]
fn diagnostics_and_capabilities_are_honest_and_bounded() {
    assert!(accepts_reply(
        &json!({"schemaVersion":1,"payload":{"type":"capabilities","screenshot":true,"colorPicker":false}})
    ));
    assert!(!accepts_reply(
        &json!({"schemaVersion":1,"payload":{"type":"capabilities","screenshot":true,"colorPicker":true}})
    ));
    for message in [
        "".to_owned(),
        "x".repeat(257),
        "terminal\u{1b}[31m".to_owned(),
    ] {
        let mut value = reply("failed");
        value["payload"]["job"]["diagnostic"]["message"] = json!(message);
        assert!(!accepts_reply(&value));
    }
}

#[test]
fn json_schemas_agree_with_runtime_for_state_invariants() {
    for (file, values, validate) in [
        (
            "capture-request-v1.schema.json",
            vec![request("begin"), request("status"), request("cancel")],
            accepts_request as fn(&Value) -> bool,
        ),
        (
            "capture-reply-v1.schema.json",
            vec![
                reply("awaitingConsent"),
                reply("capturing"),
                reply("completed"),
                reply("cancelled"),
                reply("failed"),
            ],
            accepts_reply as fn(&Value) -> bool,
        ),
    ] {
        let schema: Value =
            serde_json::from_str(&std::fs::read_to_string(format!("schemas/{file}")).unwrap())
                .unwrap();
        let validator = jsonschema::validator_for(&schema).unwrap();
        for value in values {
            assert!(validate(&value));
            assert!(validator.is_valid(&value));
            let mut unknown = value.clone();
            unknown["untrusted"] = json!(true);
            assert!(!validate(&unknown));
            assert!(!validator.is_valid(&unknown));
            if value["payload"]["type"] == "job" {
                let mut invalid = value.clone();
                invalid["payload"]["job"]["result"] = Value::Null;
                assert!(!validate(&invalid));
                assert!(!validator.is_valid(&invalid));
                let mut invalid = value;
                invalid["payload"]["job"]["state"] = json!("unknown");
                assert!(!validate(&invalid));
                assert!(!validator.is_valid(&invalid));
            }
        }
    }
}

#[test]
fn desktop_v3_remains_closed_and_legacy_capture_commands_still_validate() {
    let snapshot = include_str!("../fixtures/desktop-runtime/full-snapshot.json");
    assert!(sleepy_sdk::validate_desktop_envelope(snapshot).is_ok());
    let mut augmented: Value = serde_json::from_str(snapshot).unwrap();
    augmented["payload"]["data"]["utilities"]["captureJob"] =
        reply("capturing")["payload"]["job"].clone();
    assert!(sleepy_sdk::validate_desktop_envelope(&augmented.to_string()).is_err());
    for command in [
        json!({"type":"screenshot","data":{"outputId":"output:DP-1"}}),
        json!({"type":"pickColor"}),
    ] {
        let value = json!({"schemaVersion":3,"requestId":ID,"expectedGeneration":7,"command":{"family":"utility","command":command}});
        assert!(sleepy_sdk::validate_desktop_request(&value.to_string()).is_ok());
    }
    assert!(sleepy_sdk::validate_desktop_request(&request("begin").to_string()).is_err());
    assert!(!accepts_request(
        &serde_json::from_str::<Value>(include_str!("../fixtures/desktop-runtime/command.json"))
            .unwrap()
    ));
}

#[test]
fn malformed_replies_and_unknown_fields_are_rejected_without_panics() {
    for input in ["null", "[]", "{}", "{", "#aéabc"] {
        assert!(validate_capture_reply(input).is_err());
    }
    let mut value = reply("completed");
    value["payload"]["job"]["result"]["path"] = json!("x");
    assert!(!accepts_reply(&value));
    let mut value = reply("completed");
    value["payload"]["job"]["result"]["path"] = json!(
        "/run/user/4294967296/sleepy/captures/screenshot-018f3f4c-8af1-7f6b-bf42-1bd472868e65.png"
    );
    assert!(!accepts_reply(&value));
    let mut value = reply("capturing");
    value["payload"]["job"]["shell"] = json!("arbitrary");
    assert!(!accepts_reply(&value));
    assert!(validate_capture_request(&" ".repeat(4097)).is_err());
    assert!(validate_capture_reply(&" ".repeat(4097)).is_err());
}
