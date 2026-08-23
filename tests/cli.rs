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
