# Sleepy SDK

`sleepy-sdk` is the versioned public document contract for Sleepy Linux. It
owns the Rust document types, JSON schemas, fixtures, and validation helpers
used by other Sleepy repositories.

Every v1 document has `"schemaVersion": 1`. Unknown top-level keys are
rejected. The canonical built-in preset identifier is `builtin.sleepy`; user
presets use UUID identifiers.

## Rust API

```rust
use sleepy_sdk::{validate_preset, PresetDocument};

let preset: PresetDocument = validate_preset(json_document)?;
```

The public validators are `validate_settings`, `validate_preset`, and
`validate_plugin_manifest`. They deserialize a document and enforce the v1
contract, including safe, package-relative `.qml` plugin entrypoints.

## CLI

```sh
sleepy-contract validate settings settings.json
sleepy-contract validate preset preset.json
sleepy-contract validate plugin plugin.json
```

The command exits `0` for a valid document, `1` for an invalid or unreadable
document, and `2` for invalid command-line usage.

## Contract artifacts

- `schemas/settings.schema.json`
- `schemas/preset.schema.json`
- `schemas/plugin.schema.json`
- `fixtures/v1/`

Run the checks locally with:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## License

Licensed under GPL-3.0-only. See [LICENSE](LICENSE).
