use std::{collections::BTreeMap, fmt};

use serde::{
    de::{self, DeserializeOwned, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::{ContractError, CONTRACT_SCHEMA_VERSION};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SystemSnapshot {
    pub schema_version: u32,
    pub capabilities: BTreeMap<CapabilityId, CapabilityState>,
    pub diagnostics: BTreeMap<CapabilityId, CapabilityDiagnostic>,
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
    #[serde(rename = "session.lock")]
    SessionLock,
    #[serde(rename = "session.logout")]
    SessionLogout,
    #[serde(rename = "session.reboot")]
    SessionReboot,
    #[serde(rename = "session.powerOff")]
    SessionPowerOff,
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
    Timeout,
    Parse,
    Command,
    Busy,
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
    pub output_device: Option<String>,
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
    pub profile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaState {
    pub title: String,
    #[serde(deserialize_with = "required_option")]
    pub artist: Option<String>,
    pub playing: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SystemMutationResult {
    pub schema_version: u32,
    pub capability: CapabilityId,
    pub requested_value: SystemMutationValue,
    pub snapshot: SystemSnapshot,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SystemMutationValue {
    Boolean(bool),
    Level(f64),
    Name(String),
}

impl<'de> Deserialize<'de> for SystemMutationValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MutationValueVisitor;

        impl Visitor<'_> for MutationValueVisitor {
            type Value = SystemMutationValue;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a boolean, normalized number, or string")
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
                Ok(SystemMutationValue::Boolean(value))
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                validate_normalized(value).map_err(E::custom)?;
                Ok(SystemMutationValue::Level(value))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_f64(value as f64)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_f64(value as f64)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(SystemMutationValue::Name(value.to_owned()))
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
                Ok(SystemMutationValue::Name(value))
            }
        }

        deserializer.deserialize_any(MutationValueVisitor)
    }
}

pub fn validate_system_snapshot(input: &str) -> Result<SystemSnapshot, ContractError> {
    let snapshot: SystemSnapshot = parse_document(input)?;
    validate_snapshot(&snapshot)?;
    Ok(snapshot)
}

pub fn validate_system_mutation_result(input: &str) -> Result<SystemMutationResult, ContractError> {
    let result: SystemMutationResult = parse_document(input)?;
    validate_schema_version(result.schema_version, "system mutation result")?;
    if let SystemMutationValue::Name(value) = &result.requested_value {
        require_non_empty(value, "system mutation requestedValue")?;
    }
    validate_snapshot(&result.snapshot)?;
    Ok(result)
}

fn validate_snapshot(snapshot: &SystemSnapshot) -> Result<(), ContractError> {
    validate_schema_version(snapshot.schema_version, "system snapshot")?;
    if snapshot.capabilities.is_empty() {
        return Err(ContractError::new(
            "system snapshot capabilities must not be empty",
        ));
    }
    for diagnostic in snapshot.diagnostics.values() {
        require_non_empty(&diagnostic.message, "system snapshot diagnostic message")?;
    }
    Ok(())
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

fn validate_normalized(value: f64) -> Result<(), &'static str> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err("normalized level must be between 0.0 and 1.0")
    }
}
