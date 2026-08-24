use std::collections::{BTreeMap, BTreeSet};

use serde::{
    de::{self, DeserializeOwned},
    Deserialize, Deserializer, Serialize,
};

use crate::{ContractError, CONTRACT_SCHEMA_VERSION};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SystemSnapshot {
    pub schema_version: u32,
    pub generation: u64,
    pub capabilities: BTreeMap<CapabilityId, CapabilityState>,
    pub diagnostics: BTreeMap<CapabilityId, CapabilityDiagnostic>,
    pub session_actions: BTreeMap<SessionAction, CapabilityState>,
    #[serde(deserialize_with = "required_option")]
    pub network: Option<NetworkState>,
    #[serde(deserialize_with = "required_option")]
    pub bluetooth: Option<BluetoothState>,
    #[serde(deserialize_with = "required_option")]
    pub audio: Option<AudioState>,
    #[serde(deserialize_with = "required_option")]
    pub display: Option<DisplayState>,
    #[serde(deserialize_with = "required_option")]
    pub power: Option<PowerState>,
    #[serde(deserialize_with = "required_option")]
    pub media: Option<MediaState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CapabilityId {
    #[serde(rename = "network.enabled")]
    NetworkEnabled,
    #[serde(rename = "bluetooth.enabled")]
    BluetoothEnabled,
    #[serde(rename = "audio.volume")]
    AudioVolume,
    #[serde(rename = "audio.muted")]
    AudioMuted,
    #[serde(rename = "audio.microphoneLevel")]
    AudioMicrophoneLevel,
    #[serde(rename = "audio.microphoneMuted")]
    AudioMicrophoneMuted,
    #[serde(rename = "audio.outputDevice")]
    AudioOutputDevice,
    #[serde(rename = "display.brightness")]
    DisplayBrightness,
    #[serde(rename = "display.nightLightEnabled")]
    DisplayNightLightEnabled,
    #[serde(rename = "power.profile")]
    PowerProfile,
    #[serde(rename = "battery.status")]
    BatteryStatus,
    #[serde(rename = "media.transport")]
    MediaTransport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SessionAction {
    #[serde(rename = "lock")]
    Lock,
    #[serde(rename = "logout")]
    Logout,
    #[serde(rename = "reboot")]
    Reboot,
    #[serde(rename = "powerOff")]
    PowerOff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CapabilityState {
    Available,
    Unavailable,
    Busy,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapabilityDiagnostic {
    pub kind: CapabilityErrorKind,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CapabilityErrorKind {
    Unsupported,
    Busy,
    Timeout,
    Parse,
    Command,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkState {
    pub enabled: bool,
    #[serde(deserialize_with = "required_option")]
    pub connected_name: Option<String>,
    #[serde(deserialize_with = "required_normalized_option")]
    pub signal_level: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BluetoothState {
    pub enabled: bool,
    #[serde(deserialize_with = "required_option")]
    pub connected_device: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioState {
    #[serde(deserialize_with = "normalized")]
    pub volume: f64,
    pub muted: bool,
    #[serde(deserialize_with = "normalized")]
    pub microphone_level: f64,
    pub microphone_muted: bool,
    #[serde(deserialize_with = "required_option")]
    pub output_device_id: Option<String>,
    pub output_devices: Vec<AudioOutputDevice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioOutputDevice {
    pub id: String,
    pub label: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DisplayState {
    #[serde(deserialize_with = "required_normalized_option")]
    pub brightness: Option<f64>,
    pub night_light_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PowerState {
    #[serde(deserialize_with = "required_normalized_option")]
    pub battery_level: Option<f64>,
    #[serde(deserialize_with = "required_option")]
    pub charging: Option<bool>,
    #[serde(deserialize_with = "required_option")]
    pub current_profile: Option<PowerProfile>,
    pub available_profiles: Vec<PowerProfile>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PowerProfile {
    #[serde(rename = "power-saver")]
    PowerSaver,
    #[serde(rename = "balanced")]
    Balanced,
    #[serde(rename = "performance")]
    Performance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaState {
    pub title: String,
    #[serde(deserialize_with = "required_option")]
    pub artist: Option<String>,
    pub playing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaTransport {
    #[serde(rename = "playPause")]
    PlayPause,
    #[serde(rename = "next")]
    Next,
    #[serde(rename = "previous")]
    Previous,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "capability", content = "value", deny_unknown_fields)]
pub enum SystemMutation {
    #[serde(rename = "network.enabled")]
    NetworkEnabled(bool),
    #[serde(rename = "bluetooth.enabled")]
    BluetoothEnabled(bool),
    #[serde(rename = "audio.volume")]
    AudioVolume(#[serde(deserialize_with = "normalized")] f64),
    #[serde(rename = "audio.muted")]
    AudioMuted(bool),
    #[serde(rename = "audio.microphoneLevel")]
    AudioMicrophoneLevel(#[serde(deserialize_with = "normalized")] f64),
    #[serde(rename = "audio.microphoneMuted")]
    AudioMicrophoneMuted(bool),
    #[serde(rename = "audio.outputDevice")]
    AudioOutputDevice(#[serde(deserialize_with = "non_empty_string")] String),
    #[serde(rename = "display.brightness")]
    DisplayBrightness(#[serde(deserialize_with = "normalized")] f64),
    #[serde(rename = "display.nightLightEnabled")]
    DisplayNightLightEnabled(bool),
    #[serde(rename = "power.profile")]
    PowerProfile(PowerProfile),
    #[serde(rename = "media.transport")]
    MediaTransport(MediaTransport),
}

impl SystemMutation {
    fn capability(&self) -> CapabilityId {
        match self {
            Self::NetworkEnabled(_) => CapabilityId::NetworkEnabled,
            Self::BluetoothEnabled(_) => CapabilityId::BluetoothEnabled,
            Self::AudioVolume(_) => CapabilityId::AudioVolume,
            Self::AudioMuted(_) => CapabilityId::AudioMuted,
            Self::AudioMicrophoneLevel(_) => CapabilityId::AudioMicrophoneLevel,
            Self::AudioMicrophoneMuted(_) => CapabilityId::AudioMicrophoneMuted,
            Self::AudioOutputDevice(_) => CapabilityId::AudioOutputDevice,
            Self::DisplayBrightness(_) => CapabilityId::DisplayBrightness,
            Self::DisplayNightLightEnabled(_) => CapabilityId::DisplayNightLightEnabled,
            Self::PowerProfile(_) => CapabilityId::PowerProfile,
            Self::MediaTransport(_) => CapabilityId::MediaTransport,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SystemMutationResult {
    pub schema_version: u32,
    pub generation: u64,
    pub mutation: SystemMutation,
    pub snapshot: SystemSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionActionRequest {
    pub schema_version: u32,
    pub action: SessionAction,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionActionStatus {
    Initiated,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SessionActionResult {
    pub schema_version: u32,
    pub generation: u64,
    pub action: SessionAction,
    pub status: SessionActionStatus,
    #[serde(deserialize_with = "required_option")]
    pub diagnostic: Option<CapabilityDiagnostic>,
}

pub fn validate_system_snapshot(input: &str) -> Result<SystemSnapshot, ContractError> {
    let snapshot: SystemSnapshot = parse_document(input)?;
    validate_snapshot(&snapshot)?;
    Ok(snapshot)
}

pub fn validate_system_mutation_result(input: &str) -> Result<SystemMutationResult, ContractError> {
    let result: SystemMutationResult = parse_document(input)?;
    validate_schema_version(result.schema_version, "system mutation result")?;
    validate_snapshot(&result.snapshot)?;
    if result.generation != result.snapshot.generation {
        return Err(ContractError::new(
            "system mutation result generation must match snapshot generation",
        ));
    }
    validate_confirmed_mutation(&result.mutation, &result.snapshot)?;
    Ok(result)
}

pub fn validate_session_action_request(input: &str) -> Result<SessionActionRequest, ContractError> {
    let request: SessionActionRequest = parse_document(input)?;
    validate_schema_version(request.schema_version, "session action request")?;
    if !request.confirmed {
        return Err(ContractError::new(
            "session action request confirmed must be true",
        ));
    }
    Ok(request)
}

pub fn validate_session_action_result(input: &str) -> Result<SessionActionResult, ContractError> {
    let result: SessionActionResult = parse_document(input)?;
    validate_schema_version(result.schema_version, "session action result")?;
    validate_generation(result.generation, "session action result")?;
    match (result.status, &result.diagnostic) {
        (SessionActionStatus::Initiated, None) | (SessionActionStatus::Failed, Some(_)) => {}
        (SessionActionStatus::Initiated, Some(_)) => {
            return Err(ContractError::new(
                "initiated session action result must not contain a diagnostic",
            ));
        }
        (SessionActionStatus::Failed, None) => {
            return Err(ContractError::new(
                "failed session action result requires a diagnostic",
            ));
        }
    }
    if let Some(diagnostic) = &result.diagnostic {
        validate_diagnostic(diagnostic)?;
    }
    Ok(result)
}

fn validate_snapshot(snapshot: &SystemSnapshot) -> Result<(), ContractError> {
    validate_schema_version(snapshot.schema_version, "system snapshot")?;
    validate_generation(snapshot.generation, "system snapshot")?;
    if snapshot.capabilities.is_empty() {
        return Err(ContractError::new(
            "system snapshot capabilities must not be empty",
        ));
    }
    for diagnostic in snapshot.diagnostics.values() {
        validate_diagnostic(diagnostic)?;
    }

    let expected_actions = [
        SessionAction::Lock,
        SessionAction::Logout,
        SessionAction::Reboot,
        SessionAction::PowerOff,
    ];
    if snapshot.session_actions.len() != expected_actions.len()
        || expected_actions
            .iter()
            .any(|action| !snapshot.session_actions.contains_key(action))
    {
        return Err(ContractError::new(
            "system snapshot sessionActions must contain every session action",
        ));
    }

    if let Some(audio) = &snapshot.audio {
        validate_audio_state(audio)?;
    }
    if let Some(power) = &snapshot.power {
        validate_power_state(power)?;
    }
    Ok(())
}

fn validate_diagnostic(diagnostic: &CapabilityDiagnostic) -> Result<(), ContractError> {
    require_non_empty(&diagnostic.message, "capability diagnostic message")
}

fn validate_audio_state(audio: &AudioState) -> Result<(), ContractError> {
    let mut ids = BTreeSet::new();
    let mut default_id = None;
    for device in &audio.output_devices {
        require_non_empty(&device.id, "audio output device id")?;
        require_non_empty(&device.label, "audio output device label")?;
        if !ids.insert(device.id.as_str()) {
            return Err(ContractError::new("audio output device ids must be unique"));
        }
        if device.is_default && default_id.replace(device.id.as_str()).is_some() {
            return Err(ContractError::new(
                "audio output devices must contain exactly one default",
            ));
        }
    }

    match (&audio.output_device_id, default_id) {
        (None, None) if audio.output_devices.is_empty() => Ok(()),
        (Some(selected), Some(default)) if selected == default => Ok(()),
        _ => Err(ContractError::new(
            "audio outputDeviceId must identify the one default output device",
        )),
    }
}

fn validate_power_state(power: &PowerState) -> Result<(), ContractError> {
    let available: BTreeSet<_> = power.available_profiles.iter().copied().collect();
    if available.len() != power.available_profiles.len() {
        return Err(ContractError::new(
            "power availableProfiles must not contain duplicates",
        ));
    }
    match power.current_profile {
        None if available.is_empty() => Ok(()),
        Some(current) if available.contains(&current) => Ok(()),
        _ => Err(ContractError::new(
            "power currentProfile must be present in availableProfiles",
        )),
    }
}

fn validate_confirmed_mutation(
    mutation: &SystemMutation,
    snapshot: &SystemSnapshot,
) -> Result<(), ContractError> {
    if snapshot.capabilities.get(&mutation.capability()) != Some(&CapabilityState::Available) {
        return Err(ContractError::new(
            "system mutation result target capability must be available",
        ));
    }

    let confirmed = match mutation {
        SystemMutation::NetworkEnabled(value) => snapshot
            .network
            .as_ref()
            .is_some_and(|state| state.enabled == *value),
        SystemMutation::BluetoothEnabled(value) => snapshot
            .bluetooth
            .as_ref()
            .is_some_and(|state| state.enabled == *value),
        SystemMutation::AudioVolume(value) => snapshot
            .audio
            .as_ref()
            .is_some_and(|state| state.volume == *value),
        SystemMutation::AudioMuted(value) => snapshot
            .audio
            .as_ref()
            .is_some_and(|state| state.muted == *value),
        SystemMutation::AudioMicrophoneLevel(value) => snapshot
            .audio
            .as_ref()
            .is_some_and(|state| state.microphone_level == *value),
        SystemMutation::AudioMicrophoneMuted(value) => snapshot
            .audio
            .as_ref()
            .is_some_and(|state| state.microphone_muted == *value),
        SystemMutation::AudioOutputDevice(value) => {
            snapshot
                .audio
                .as_ref()
                .and_then(|state| state.output_device_id.as_deref())
                == Some(value.as_str())
        }
        SystemMutation::DisplayBrightness(value) => {
            snapshot.display.as_ref().and_then(|state| state.brightness) == Some(*value)
        }
        SystemMutation::DisplayNightLightEnabled(value) => snapshot
            .display
            .as_ref()
            .is_some_and(|state| state.night_light_enabled == *value),
        SystemMutation::PowerProfile(value) => {
            snapshot
                .power
                .as_ref()
                .and_then(|state| state.current_profile)
                == Some(*value)
        }
        SystemMutation::MediaTransport(_) => snapshot.media.is_some(),
    };

    if confirmed {
        Ok(())
    } else {
        Err(ContractError::new(
            "system mutation result snapshot does not confirm the mutation",
        ))
    }
}

fn parse_document<T: DeserializeOwned>(input: &str) -> Result<T, ContractError> {
    serde_json::from_str(input).map_err(|error| ContractError::new(error.to_string()))
}

fn validate_schema_version(version: u32, name: &str) -> Result<(), ContractError> {
    if version == CONTRACT_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(ContractError::new(format!(
            "{name} schemaVersion must be {CONTRACT_SCHEMA_VERSION}"
        )))
    }
}

fn validate_generation(generation: u64, name: &str) -> Result<(), ContractError> {
    if generation == 0 {
        Err(ContractError::new(format!(
            "{name} generation must be positive"
        )))
    } else {
        Ok(())
    }
}

fn require_non_empty(value: &str, name: &str) -> Result<(), ContractError> {
    if value.trim().is_empty() {
        Err(ContractError::new(format!("{name} must not be empty")))
    } else {
        Ok(())
    }
}

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}

fn normalized<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = f64::deserialize(deserializer)?;
    validate_normalized(value).map_err(de::Error::custom)?;
    Ok(value)
}

fn required_normalized_option<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<f64>::deserialize(deserializer)?;
    if let Some(value) = value {
        validate_normalized(value).map_err(de::Error::custom)?;
    }
    Ok(value)
}

fn non_empty_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value.trim().is_empty() {
        Err(de::Error::custom("value must not be empty"))
    } else {
        Ok(value)
    }
}

fn validate_normalized(value: f64) -> Result<(), &'static str> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err("normalized level must be between 0.0 and 1.0")
    }
}
