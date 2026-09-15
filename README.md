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
# Opt-in capture v1

`CaptureRequest` / `CaptureReply` describe a separate endpoint owned by the
session daemon. They do not add fields or variants to the closed desktop v3
stream. Existing v3 `Screenshot` and `PickColor` commands keep their semantics.
The SDK defines the contract; it does not itself capture pixels or obtain consent.

```json
{"schemaVersion":1,"command":{"type":"begin","jobId":"018f3f4c-8af1-7f6b-bf42-1bd472868e65","outputId":"output:DP-1"}}
```

`status` and `cancel` take the same `jobId`; `capabilities` takes no arguments.
Replies contain a `payload` tagged `job`, `capabilities`, or `error`. A job
contains `jobId`, `outputId`, and `state`. States are `awaitingConsent`,
`capturing`, `completed`, `cancelled`, and `failed`. A successful begin reply
reports the operation's state, never falsely promises a completed screenshot.
The client polls status while the job is active and stops at a terminal state.
No subscription or human interaction blocks the desktop v3 command mutex.

Only `completed` contains `result`: PNG `path`, `mimeType`, `width`, and `height`.
Only `failed` contains `diagnostic`: a fixed error `code` and a bounded, plain-text
`message`. Error replies carry that same diagnostic. Optional fields must be
omitted, not null. Initial capabilities advertise `colorPicker: false`.

The provider owns at most one active job per user, limits consent waiting to
120 seconds, and retains only bounded terminal job history. Duplicate begin
with the same ID and target must not create another prompt; reusing that ID for
a different target is rejected. Cancellation and a terminal job's status are
idempotent while its record is retained. An unknown or evicted ID returns
`notFound`. Backend shutdown cancels and reaps owned work.

The result is a temporary session file under
`/run/user/<uid>/sleepy/captures/screenshot-<jobId>.png`, retained until session
end; consumers must explicitly export it to keep it. History eviction must not
delete another job's or an unrelated file. The SDK checks canonical UUIDs,
output names, filename/ID agreement, path syntax, dimensions, and state/result
invariants. The provider must additionally verify the actual user's runtime
directory, no symlink escape, file ownership, PNG bytes/dimensions, and genuine
user consent before publishing completion. Clients must correlate returned
job and output IDs with their request. Each JSON frame is at most 4096 bytes.

Public schemas are `schemas/capture-request-v1.schema.json` and
`schemas/capture-reply-v1.schema.json`; cross-field filename checks and runtime
filesystem checks are explicitly outside JSON Schema's structural guarantees.
