use std::process::Command;

fn sleepy_contract() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sleepy-contract"))
}

#[test]
fn exits_zero_for_a_valid_document() {
    let status = sleepy_contract()
        .args(["validate", "settings", "fixtures/v1/settings/valid.json"])
        .status()
        .expect("CLI should run");

    assert_eq!(status.code(), Some(0));
}

#[test]
fn exits_one_for_an_invalid_document() {
    let status = sleepy_contract()
        .args([
            "validate",
            "plugin",
            "fixtures/v1/plugin/invalid-unsafe-entrypoint.json",
        ])
        .status()
        .expect("CLI should run");

    assert_eq!(status.code(), Some(1));
}

#[test]
fn validates_system_documents() {
    let valid_status = sleepy_contract()
        .args(["validate", "system", "fixtures/v1/system/valid.json"])
        .status()
        .expect("CLI should run");
    let invalid_status = sleepy_contract()
        .args([
            "validate",
            "system",
            "fixtures/v1/system/invalid-unknown-field.json",
        ])
        .status()
        .expect("CLI should run");
    let mutation_status = sleepy_contract()
        .args([
            "validate",
            "system",
            "fixtures/v1/system/valid-mutation.json",
        ])
        .status()
        .expect("CLI should run");
    let request_status = sleepy_contract()
        .args([
            "validate",
            "system",
            "fixtures/v1/system/valid-session-request.json",
        ])
        .status()
        .expect("CLI should run");
    let action_result_status = sleepy_contract()
        .args([
            "validate",
            "system",
            "fixtures/v1/system/valid-session-result-initiated.json",
        ])
        .status()
        .expect("CLI should run");

    assert_eq!(valid_status.code(), Some(0));
    assert_eq!(invalid_status.code(), Some(1));
    assert_eq!(mutation_status.code(), Some(0));
    assert_eq!(request_status.code(), Some(0));
    assert_eq!(action_result_status.code(), Some(0));
}

#[test]
fn rejects_a_preset_that_collides_with_the_packaged_recovery_chord() {
    let output = sleepy_contract()
        .args([
            "validate",
            "preset",
            "fixtures/v1/preset/invalid-reserved-binding.json",
        ])
        .output()
        .expect("CLI should run");

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Reserved"));
}

#[test]
fn reports_mutation_specific_system_document_errors() {
    let output = sleepy_contract()
        .args([
            "validate",
            "system",
            "fixtures/v1/system/invalid-mutation-missing-requested-value.json",
        ])
        .output()
        .expect("CLI should run");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr.contains("missing field `mutation`"),
        "unexpected diagnostic: {stderr}"
    );
}

#[test]
fn rejects_mutation_results_without_available_coherent_readback() {
    let output = sleepy_contract()
        .args([
            "validate",
            "system",
            "fixtures/v1/system/invalid-mutation-unavailable-media.json",
        ])
        .output()
        .expect("CLI should run");

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn reports_missing_status_for_a_malformed_session_result() {
    let output = sleepy_contract()
        .args([
            "validate",
            "system",
            "fixtures/v1/system/invalid-session-result-missing-status.json",
        ])
        .output()
        .expect("CLI should run");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr.contains("missing field `status`"),
        "unexpected diagnostic: {stderr}"
    );
}

#[test]
fn rejects_modifier_only_preset_chords() {
    for path in [
        "fixtures/v1/preset/invalid-keyless-modifier.json",
        "fixtures/v1/preset/invalid-keyless-modifier-chain.json",
    ] {
        let output = sleepy_contract()
            .args(["validate", "preset", path])
            .output()
            .expect("CLI should run");

        assert_eq!(output.status.code(), Some(1), "CLI must reject {path}");
    }
}
