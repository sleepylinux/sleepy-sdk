use serde::{Deserialize, Serialize};

use crate::ContractError;

pub const WIRE_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventEnvelope {
    pub schema_version: u32,
    pub generation: u64,
    pub event_id: String,
    pub emitted_at: String,
    pub cause: EventCause,
    pub payload: SessionEvent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventCause {
    pub kind: EventCauseKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EventCauseKind {
    External,
    Request,
    Replay,
    Lifecycle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum SessionEvent {
    FullSnapshot(RuntimeSnapshot),
    CapabilityUpdate(CapabilityRecord),
    Notification(NotificationEvent),
    Provider(ProviderEvent),
    Niri(NiriEvent),
    Theme(ThemeEvent),
    Lifecycle(LifecycleEvent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeSnapshot {
    pub capabilities: Vec<CapabilityRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focused_output_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityRecord {
    pub id: RuntimeCapabilityId,
    pub status: CapabilityAvailability,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<CapabilityValue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<CapabilityFailure>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuntimeCapabilityId {
    Network,
    Bluetooth,
    Audio,
    Battery,
    Brightness,
    PowerProfile,
    Media,
    NightLight,
    Niri,
    Resources,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CapabilityAvailability {
    Available,
    Unavailable,
    Unsupported,
    PermissionDenied,
    Timeout,
    Parse,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityFailure {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum CapabilityValue {
    Network(NetworkRuntimeState),
    Bluetooth(BluetoothRuntimeState),
    Audio(AudioRuntimeState),
    Battery(BatteryRuntimeState),
    Brightness(BrightnessRuntimeState),
    PowerProfile(PowerProfileRuntimeState),
    Media(MediaRuntimeState),
    NightLight(NightLightRuntimeState),
    Niri(NiriRuntimeState),
    Resources(ResourceRuntimeState),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkRuntimeState {
    pub wifi_enabled: bool,
    pub ethernet_connected: bool,
    pub connectivity: Connectivity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_connection_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Connectivity {
    Unknown,
    None,
    Portal,
    Limited,
    Full,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BluetoothRuntimeState {
    pub powered: bool,
    pub connected_device_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioRuntimeState {
    pub output_level: f64,
    pub output_muted: bool,
    pub input_level: f64,
    pub input_muted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_output_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BatteryRuntimeState {
    pub percentage: u8,
    pub charging: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds_remaining: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrightnessRuntimeState {
    pub level: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PowerProfileRuntimeState {
    pub active: String,
    pub available: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaRuntimeState {
    pub player_id: String,
    pub title: String,
    pub artist: String,
    pub playing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NightLightRuntimeState {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NiriRuntimeState {
    pub output_ids: Vec<String>,
    pub workspace_ids: Vec<u64>,
    pub window_ids: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceRuntimeState {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub load_one: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotificationEvent {
    pub notification_id: u64,
    pub change: NotificationChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NotificationChange {
    Added,
    Updated,
    Archived,
    ActionExpired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderEvent {
    pub provider_id: String,
    pub online: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NiriEvent {
    pub focused_output_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThemeEvent {
    pub theme_id: String,
    pub applied: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LifecycleEvent {
    pub state: LifecycleState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LifecycleState {
    Ready,
    Stopping,
    Reconciled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MutationRequest {
    pub schema_version: u32,
    pub request_id: String,
    pub expected_generation: u64,
    pub command: DaemonCommand,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum DaemonCommand {
    SetDnd { enabled: bool },
    SetCapability { mutation: crate::SystemMutation },
    FocusWindow { window_id: u64 },
    CloseWindow { window_id: u64 },
    FocusWorkspace { workspace_id: u64 },
    ApplyTheme { theme_id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MutationResult {
    pub schema_version: u32,
    pub request_id: String,
    pub generation: u64,
    pub status: MutationStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmed_event: Option<EventEnvelope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<MutationFailure>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MutationStatus {
    Confirmed,
    Rejected,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MutationFailure {
    pub code: String,
    pub message: String,
}

pub fn validate_event_envelope(input: &str) -> Result<EventEnvelope, ContractError> {
    let envelope: EventEnvelope = parse(input, "event envelope")?;
    validate_envelope(&envelope)?;
    Ok(envelope)
}

pub fn validate_mutation_request(input: &str) -> Result<MutationRequest, ContractError> {
    let request: MutationRequest = parse(input, "mutation request")?;
    validate_wire_version(request.schema_version, "mutation request")?;
    validate_uuid(&request.request_id, "mutation request requestId")?;
    require_positive(request.expected_generation, "mutation expectedGeneration")?;
    validate_daemon_command(&request.command)?;
    Ok(request)
}

fn validate_daemon_command(command: &DaemonCommand) -> Result<(), ContractError> {
    match command {
        DaemonCommand::FocusWindow { window_id } | DaemonCommand::CloseWindow { window_id } => {
            require_positive(*window_id, "mutation windowId")?;
        }
        DaemonCommand::FocusWorkspace { workspace_id } => {
            require_positive(*workspace_id, "mutation workspaceId")?;
        }
        DaemonCommand::ApplyTheme { theme_id } if theme_id.trim().is_empty() => {
            return Err(ContractError::new("mutation themeId must not be empty"));
        }
        DaemonCommand::SetDnd { .. }
        | DaemonCommand::SetCapability { .. }
        | DaemonCommand::ApplyTheme { .. } => {}
    }
    Ok(())
}

pub fn validate_mutation_result(input: &str) -> Result<MutationResult, ContractError> {
    let result: MutationResult = parse(input, "mutation result")?;
    validate_wire_version(result.schema_version, "mutation result")?;
    validate_uuid(&result.request_id, "mutation result requestId")?;
    require_positive(result.generation, "mutation result generation")?;

    match result.status {
        MutationStatus::Confirmed => {
            let event = result.confirmed_event.as_ref().ok_or_else(|| {
                ContractError::new("confirmed mutation result requires confirmedEvent")
            })?;
            if result.error.is_some() {
                return Err(ContractError::new(
                    "confirmed mutation result cannot contain an error",
                ));
            }
            validate_envelope(event)?;
            if event.generation != result.generation {
                return Err(ContractError::new(
                    "mutation result generation must match confirmedEvent generation",
                ));
            }
            if event.cause.kind != EventCauseKind::Request
                || event.cause.request_id.as_deref() != Some(result.request_id.as_str())
            {
                return Err(ContractError::new(
                    "confirmed mutation event must name the same requestId",
                ));
            }
        }
        MutationStatus::Rejected | MutationStatus::Unknown => {
            if result.confirmed_event.is_some() || result.error.is_none() {
                return Err(ContractError::new(
                    "unconfirmed mutation result requires only an error",
                ));
            }
            let error = result.error.as_ref().expect("checked above");
            if error.code.trim().is_empty() || error.message.trim().is_empty() {
                return Err(ContractError::new(
                    "mutation error code and message must not be empty",
                ));
            }
        }
    }

    Ok(result)
}

pub fn validate_mutation_exchange(
    request_input: &str,
    result_input: &str,
) -> Result<(MutationRequest, MutationResult), ContractError> {
    let request = validate_mutation_request(request_input)?;
    let result = validate_mutation_result(result_input)?;
    if request.request_id != result.request_id {
        return Err(ContractError::new(
            "mutation request and result requestId must match",
        ));
    }
    if result.status == MutationStatus::Confirmed
        && result.generation <= request.expected_generation
    {
        return Err(ContractError::new(
            "confirmed mutation generation must follow expectedGeneration",
        ));
    }
    Ok((request, result))
}

fn validate_envelope(envelope: &EventEnvelope) -> Result<(), ContractError> {
    validate_wire_version(envelope.schema_version, "event envelope")?;
    require_positive(envelope.generation, "event generation")?;
    validate_uuid(&envelope.event_id, "eventId")?;
    if !is_utc_timestamp(&envelope.emitted_at) {
        return Err(ContractError::new("emittedAt must be a UTC timestamp"));
    }

    match envelope.cause.kind {
        EventCauseKind::Request => validate_uuid(
            envelope
                .cause
                .request_id
                .as_deref()
                .ok_or_else(|| ContractError::new("request cause requires requestId"))?,
            "event cause requestId",
        )?,
        _ if envelope.cause.request_id.is_some() => {
            return Err(ContractError::new(
                "only a request cause may contain requestId",
            ));
        }
        _ => {}
    }

    match &envelope.payload {
        SessionEvent::FullSnapshot(snapshot) => {
            if snapshot.capabilities.len() != ALL_RUNTIME_CAPABILITY_IDS.len() {
                return Err(ContractError::new(
                    "full snapshot requires every capability exactly once",
                ));
            }
            let mut ids = std::collections::BTreeSet::new();
            for capability in &snapshot.capabilities {
                if !ids.insert(capability.id) {
                    return Err(ContractError::new(
                        "full snapshot contains a duplicate capability",
                    ));
                }
                validate_capability(capability)?;
            }
            if ids.into_iter().ne(ALL_RUNTIME_CAPABILITY_IDS) {
                return Err(ContractError::new(
                    "full snapshot capability registry is incomplete",
                ));
            }
        }
        SessionEvent::CapabilityUpdate(capability) => validate_capability(capability)?,
        SessionEvent::Notification(notification) => {
            require_positive(
                notification.notification_id,
                "notification event notificationId",
            )?;
        }
        SessionEvent::Provider(provider) => {
            if provider.provider_id.trim().is_empty() {
                return Err(ContractError::new(
                    "provider event providerId must not be empty",
                ));
            }
        }
        SessionEvent::Theme(theme) => {
            if theme.theme_id.trim().is_empty() {
                return Err(ContractError::new("theme event themeId must not be empty"));
            }
        }
        SessionEvent::Niri(_) | SessionEvent::Lifecycle(_) => {}
    }
    Ok(())
}

fn validate_capability(capability: &CapabilityRecord) -> Result<(), ContractError> {
    match capability.status {
        CapabilityAvailability::Available => {
            if capability.value.is_none() || capability.diagnostic.is_some() {
                return Err(ContractError::new(
                    "available capability requires only a typed value",
                ));
            }
            let value = capability.value.as_ref().expect("checked above");
            let matches_id = matches!(
                (capability.id, value),
                (RuntimeCapabilityId::Network, CapabilityValue::Network(_))
                    | (
                        RuntimeCapabilityId::Bluetooth,
                        CapabilityValue::Bluetooth(_)
                    )
                    | (RuntimeCapabilityId::Audio, CapabilityValue::Audio(_))
                    | (RuntimeCapabilityId::Battery, CapabilityValue::Battery(_))
                    | (
                        RuntimeCapabilityId::Brightness,
                        CapabilityValue::Brightness(_)
                    )
                    | (
                        RuntimeCapabilityId::PowerProfile,
                        CapabilityValue::PowerProfile(_)
                    )
                    | (RuntimeCapabilityId::Media, CapabilityValue::Media(_))
                    | (
                        RuntimeCapabilityId::NightLight,
                        CapabilityValue::NightLight(_)
                    )
                    | (RuntimeCapabilityId::Niri, CapabilityValue::Niri(_))
                    | (
                        RuntimeCapabilityId::Resources,
                        CapabilityValue::Resources(_)
                    )
            );
            if !matches_id {
                return Err(ContractError::new(
                    "capability id must match its typed value",
                ));
            }
            validate_capability_value(value)?;
        }
        _ => {
            if capability.value.is_some() || capability.diagnostic.is_none() {
                return Err(ContractError::new(
                    "unavailable capability requires only a diagnostic",
                ));
            }
            validate_non_empty(
                &capability
                    .diagnostic
                    .as_ref()
                    .expect("checked above")
                    .message,
                "capability diagnostic message",
            )?;
        }
    }
    Ok(())
}

fn validate_capability_value(value: &CapabilityValue) -> Result<(), ContractError> {
    match value {
        CapabilityValue::Network(state) => {
            if let Some(id) = &state.active_connection_id {
                validate_non_empty(id, "network activeConnectionId")?;
            }
        }
        CapabilityValue::Bluetooth(state) => {
            validate_unique_non_empty(&state.connected_device_ids, "Bluetooth device id")?;
        }
        CapabilityValue::Audio(state) => {
            validate_normalized(state.output_level, "audio outputLevel")?;
            validate_normalized(state.input_level, "audio inputLevel")?;
            if let Some(id) = &state.default_output_id {
                validate_non_empty(id, "audio defaultOutputId")?;
            }
        }
        CapabilityValue::Battery(state) if state.percentage > 100 => {
            return Err(ContractError::new(
                "battery percentage must be between 0 and 100",
            ));
        }
        CapabilityValue::Brightness(state) => {
            validate_normalized(state.level, "brightness level")?;
        }
        CapabilityValue::PowerProfile(state) => {
            validate_non_empty(&state.active, "active power profile")?;
            validate_unique_non_empty(&state.available, "available power profile")?;
            if !state.available.contains(&state.active) {
                return Err(ContractError::new("active power profile must be available"));
            }
        }
        CapabilityValue::Media(state) => {
            validate_non_empty(&state.player_id, "media playerId")?;
        }
        CapabilityValue::Niri(state) => {
            validate_unique_non_empty(&state.output_ids, "Niri output id")?;
            validate_unique_positive(&state.workspace_ids, "Niri workspace id")?;
            validate_unique_positive(&state.window_ids, "Niri window id")?;
        }
        CapabilityValue::Resources(state) => {
            validate_normalized(state.cpu_usage, "resource cpuUsage")?;
            validate_normalized(state.memory_usage, "resource memoryUsage")?;
            if !state.load_one.is_finite() || state.load_one < 0.0 {
                return Err(ContractError::new(
                    "resource loadOne must be finite and non-negative",
                ));
            }
        }
        CapabilityValue::Battery(_) | CapabilityValue::NightLight(_) => {}
    }
    Ok(())
}

const ALL_RUNTIME_CAPABILITY_IDS: [RuntimeCapabilityId; 10] = [
    RuntimeCapabilityId::Network,
    RuntimeCapabilityId::Bluetooth,
    RuntimeCapabilityId::Audio,
    RuntimeCapabilityId::Battery,
    RuntimeCapabilityId::Brightness,
    RuntimeCapabilityId::PowerProfile,
    RuntimeCapabilityId::Media,
    RuntimeCapabilityId::NightLight,
    RuntimeCapabilityId::Niri,
    RuntimeCapabilityId::Resources,
];

fn validate_normalized(value: f64, name: &str) -> Result<(), ContractError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(ContractError::new(format!("{name} must be normalized")))
    }
}

fn validate_non_empty(value: &str, name: &str) -> Result<(), ContractError> {
    if value.is_empty() || value.trim() != value {
        Err(ContractError::new(format!(
            "{name} must be non-empty and unpadded"
        )))
    } else {
        Ok(())
    }
}

fn validate_unique_non_empty(values: &[String], name: &str) -> Result<(), ContractError> {
    let mut unique = std::collections::BTreeSet::new();
    for value in values {
        validate_non_empty(value, name)?;
        if !unique.insert(value) {
            return Err(ContractError::new(format!("{name} must be unique")));
        }
    }
    Ok(())
}

fn validate_unique_positive(values: &[u64], name: &str) -> Result<(), ContractError> {
    let mut unique = std::collections::BTreeSet::new();
    if values
        .iter()
        .any(|value| *value == 0 || !unique.insert(*value))
    {
        Err(ContractError::new(format!(
            "{name} must be positive and unique"
        )))
    } else {
        Ok(())
    }
}

fn validate_wire_version(version: u32, name: &str) -> Result<(), ContractError> {
    if version == WIRE_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(ContractError::new(format!(
            "unsupported {name} schema version: {version}"
        )))
    }
}

fn require_positive(value: u64, name: &str) -> Result<(), ContractError> {
    if value > 0 {
        Ok(())
    } else {
        Err(ContractError::new(format!("{name} must be positive")))
    }
}

fn validate_uuid(value: &str, name: &str) -> Result<(), ContractError> {
    let parsed = uuid::Uuid::parse_str(value)
        .map_err(|_| ContractError::new(format!("{name} must be a canonical UUID")))?;
    if parsed.hyphenated().to_string() != value {
        return Err(ContractError::new(format!(
            "{name} must be a canonical UUID"
        )));
    }
    Ok(())
}

fn is_utc_timestamp(value: &str) -> bool {
    if value.len() != 20 {
        return false;
    }
    let bytes = value.as_bytes();
    if bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    let number = |start: usize, end: usize| value[start..end].parse::<u32>().ok();
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) = (
        number(0, 4),
        number(5, 7),
        number(8, 10),
        number(11, 13),
        number(14, 16),
        number(17, 19),
    ) else {
        return false;
    };
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    day > 0 && day <= max_day && hour < 24 && minute < 60 && second < 60
}

fn parse<T: for<'de> Deserialize<'de>>(input: &str, name: &str) -> Result<T, ContractError> {
    serde_json::from_str(input)
        .map_err(|error| ContractError::new(format!("invalid {name}: {error}")))
}
