use std::{env, fs, process};

use sleepy_sdk::{
    validate_plugin_manifest, validate_preset, validate_session_action_request,
    validate_session_action_result, validate_settings, validate_system_mutation_result,
    validate_system_snapshot, ContractError,
};

fn main() {
    process::exit(run(env::args().skip(1)));
}

fn run(mut arguments: impl Iterator<Item = String>) -> i32 {
    let (Some(command), Some(document_kind), Some(path), None) = (
        arguments.next(),
        arguments.next(),
        arguments.next(),
        arguments.next(),
    ) else {
        print_usage();
        return 2;
    };

    if command != "validate" {
        print_usage();
        return 2;
    }

    let document = match fs::read_to_string(&path) {
        Ok(document) => document,
        Err(error) => {
            eprintln!("failed to read {path}: {error}");
            return 1;
        }
    };

    let validation: Result<(), ContractError> = match document_kind.as_str() {
        "settings" => validate_settings(&document).map(|_| ()),
        "preset" => validate_preset(&document).map(|_| ()),
        "plugin" => validate_plugin_manifest(&document).map(|_| ()),
        "system" => validate_system_document(&document),
        _ => {
            print_usage();
            return 2;
        }
    };

    match validation {
        Ok(()) => {
            println!("valid {document_kind} document: {path}");
            0
        }
        Err(error) => {
            eprintln!("invalid {document_kind} document {path}: {error}");
            1
        }
    }
}

fn print_usage() {
    eprintln!("usage: sleepy-contract validate <settings|preset|plugin|system> <path>");
}

fn validate_system_document(document: &str) -> Result<(), ContractError> {
    let shape = serde_json::from_str::<serde_json::Value>(document)
        .ok()
        .and_then(|value| {
            value.as_object().map(|object| {
                if object.contains_key("mutation") || object.contains_key("snapshot") {
                    "mutation"
                } else if object.contains_key("status") {
                    "sessionResult"
                } else if object.contains_key("action") || object.contains_key("confirmed") {
                    "sessionRequest"
                } else {
                    "snapshot"
                }
            })
        })
        .unwrap_or("snapshot");

    match shape {
        "mutation" => validate_system_mutation_result(document).map(|_| ()),
        "sessionResult" => validate_session_action_result(document).map(|_| ()),
        "sessionRequest" => validate_session_action_request(document).map(|_| ()),
        _ => validate_system_snapshot(document).map(|_| ()),
    }
}
