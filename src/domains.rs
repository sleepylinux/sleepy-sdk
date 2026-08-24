use serde::{Deserialize, Serialize};

use crate::{CapabilityAvailability, CapabilityFailure, ContractError, WIRE_SCHEMA_VERSION};

pub const DURABLE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotificationDocument {
    pub schema_version: u32,
    pub id: u64,
    pub application_id: String,
    pub summary: String,
    pub body: String,
    pub urgency: NotificationUrgency,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    pub read: bool,
    pub archived: bool,
    pub actions: Vec<NotificationAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NotificationUrgency {
    Low,
    Normal,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NotificationAction {
    pub id: String,
    pub label: String,
    pub state: NotificationActionState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NotificationActionState {
    Available,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThemeDocument {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub origin: ThemeOrigin,
    pub appearance: ThemeAppearance,
    pub effects: ThemeEffects,
    pub reduced_motion: bool,
    pub opaque_fallback: bool,
    pub colors: SemanticColors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemeOrigin {
    Builtin,
    User,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemeAppearance {
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ThemeEffects {
    Full,
    Reduced,
    None,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticColors {
    pub background: String,
    pub surface: String,
    pub text_primary: String,
    pub text_secondary: String,
    pub accent: String,
    pub control: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HardwareCapabilitySnapshot {
    pub schema_version: u32,
    pub generation: u64,
    pub devices: Vec<HardwareDeviceCapability>,
    pub fixture: FixtureMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HardwareDeviceCapability {
    pub id: DeviceIdentifier,
    pub capability: HardwareCapability,
    pub status: CapabilityAvailability,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<CapabilityFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeviceIdentifier {
    pub kind: DeviceKind,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DeviceKind {
    Cpu,
    Gpu,
    Battery,
    Backlight,
    BluetoothAdapter,
    AudioDevice,
    NetworkDevice,
    Touchpad,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HardwareCapability {
    Graphics,
    Battery,
    Brightness,
    Bluetooth,
    Audio,
    Network,
    Touchpad,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FixtureMetadata {
    pub format_version: u32,
    pub recorded_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstallationProfile {
    pub schema_version: u32,
    pub profile_id: String,
    pub machine: MachineProfile,
    pub first_boot: FirstBootSnapshot,
    pub rollback: RollbackSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MachineProfile {
    pub host_name: String,
    pub system: MachineSystem,
    pub desktop_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineSystem {
    #[serde(rename = "x86_64-linux")]
    X86_64Linux,
    #[serde(rename = "aarch64-linux")]
    Aarch64Linux,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FirstBootSnapshot {
    pub state: FirstBootState,
    pub generation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FirstBootState {
    Pending,
    Applying,
    Ready,
    Failed,
    RolledBack,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RollbackSnapshot {
    pub current_generation: u64,
    pub available_generations: Vec<u64>,
}

pub trait InstallerProvider {
    fn inspect(&self) -> Result<InstallationProfile, ContractError>;
    fn apply_profile(
        &self,
        profile: &InstallationProfile,
    ) -> Result<FirstBootSnapshot, ContractError>;
    fn rollback(&self, generation: u64) -> Result<FirstBootSnapshot, ContractError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProviderSnapshot {
    pub schema_version: u32,
    pub provider_id: String,
    pub kind: ProviderKind,
    pub status: ProviderStatus,
    pub cache: CacheStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<CapabilityFailure>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderKind {
    Calendar,
    Weather,
    Geocoding,
    Secret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderStatus {
    Online,
    Offline,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CacheStatus {
    Fresh,
    Stale,
    Missing,
}

pub fn validate_notification_document(input: &str) -> Result<NotificationDocument, ContractError> {
    let document: NotificationDocument = parse(input, "notification")?;
    require_wire_version(document.schema_version, "notification")?;
    require_positive(document.id, "notification id")?;
    require_non_empty(&document.application_id, "notification applicationId")?;
    require_non_empty(&document.summary, "notification summary")?;
    require_timestamp(&document.created_at, "notification createdAt")?;
    for action in &document.actions {
        require_non_empty(&action.id, "notification action id")?;
        require_non_empty(&action.label, "notification action label")?;
    }
    Ok(document)
}

pub fn validate_theme_document(input: &str) -> Result<ThemeDocument, ContractError> {
    let document: ThemeDocument = parse(input, "theme")?;
    require_durable_version(document.schema_version, "theme")?;
    require_non_empty(&document.name, "theme name")?;
    match document.origin {
        ThemeOrigin::User => require_uuid(&document.id, "user theme id")?,
        ThemeOrigin::Builtin => require_non_empty(&document.id, "builtin theme id")?,
    }

    let colors = &document.colors;
    for (name, color) in [
        ("background", &colors.background),
        ("surface", &colors.surface),
        ("textPrimary", &colors.text_primary),
        ("textSecondary", &colors.text_secondary),
        ("accent", &colors.accent),
        ("control", &colors.control),
    ] {
        parse_color(color)
            .map_err(|_| ContractError::new(format!("theme {name} must be #RRGGBB")))?;
    }
    for (name, foreground, minimum) in [
        ("textPrimary/background", &colors.text_primary, 4.5),
        ("textPrimary/surface", &colors.text_primary, 4.5),
        ("textSecondary/background", &colors.text_secondary, 4.5),
        ("textSecondary/surface", &colors.text_secondary, 4.5),
        ("accent/background", &colors.accent, 3.0),
        ("control/background", &colors.control, 3.0),
    ] {
        let background = if name.ends_with("surface") {
            &colors.surface
        } else {
            &colors.background
        };
        if contrast_ratio(foreground, background)? < minimum {
            return Err(ContractError::new(format!(
                "theme {name} contrast must be at least {minimum}:1"
            )));
        }
    }
    Ok(document)
}

pub fn validate_hardware_capability_snapshot(
    input: &str,
) -> Result<HardwareCapabilitySnapshot, ContractError> {
    let snapshot: HardwareCapabilitySnapshot = parse(input, "hardware capability snapshot")?;
    require_durable_version(snapshot.schema_version, "hardware capability snapshot")?;
    require_positive(snapshot.generation, "hardware generation")?;
    if snapshot.fixture.format_version != DURABLE_SCHEMA_VERSION {
        return Err(ContractError::new(
            "unsupported hardware fixture format version",
        ));
    }
    require_timestamp(&snapshot.fixture.recorded_at, "hardware fixture recordedAt")?;
    for device in &snapshot.devices {
        require_non_empty(&device.id.value, "hardware device id")?;
        let coherent = matches!(
            (device.id.kind, device.capability),
            (
                DeviceKind::Cpu | DeviceKind::Gpu,
                HardwareCapability::Graphics
            ) | (DeviceKind::Battery, HardwareCapability::Battery)
                | (DeviceKind::Backlight, HardwareCapability::Brightness)
                | (DeviceKind::BluetoothAdapter, HardwareCapability::Bluetooth)
                | (DeviceKind::AudioDevice, HardwareCapability::Audio)
                | (DeviceKind::NetworkDevice, HardwareCapability::Network)
                | (DeviceKind::Touchpad, HardwareCapability::Touchpad)
        );
        if !coherent {
            return Err(ContractError::new(
                "hardware device kind and capability must match",
            ));
        }
        match device.status {
            CapabilityAvailability::Available if device.diagnostic.is_some() => {
                return Err(ContractError::new(
                    "available hardware capability cannot have a diagnostic",
                ));
            }
            CapabilityAvailability::Available => {}
            _ if device.diagnostic.is_none() => {
                return Err(ContractError::new(
                    "unavailable hardware capability requires a diagnostic",
                ));
            }
            _ => {}
        }
        if let Some(diagnostic) = &device.diagnostic {
            require_non_empty(&diagnostic.message, "hardware diagnostic message")?;
        }
    }
    Ok(snapshot)
}

pub fn validate_installation_profile(input: &str) -> Result<InstallationProfile, ContractError> {
    let profile: InstallationProfile = parse(input, "installation profile")?;
    require_durable_version(profile.schema_version, "installation profile")?;
    require_non_empty(&profile.profile_id, "installation profile id")?;
    require_non_empty(&profile.machine.host_name, "machine hostName")?;
    require_positive(profile.first_boot.generation, "first boot generation")?;
    require_positive(
        profile.rollback.current_generation,
        "rollback currentGeneration",
    )?;
    let unique_generations = profile
        .rollback
        .available_generations
        .iter()
        .copied()
        .collect::<std::collections::BTreeSet<_>>();
    if profile.rollback.available_generations.is_empty()
        || !profile
            .rollback
            .available_generations
            .contains(&profile.rollback.current_generation)
        || profile.rollback.available_generations.contains(&0)
        || unique_generations.len() != profile.rollback.available_generations.len()
    {
        return Err(ContractError::new(
            "rollback generations must contain the positive current generation",
        ));
    }
    Ok(profile)
}

pub fn validate_provider_snapshot(input: &str) -> Result<ProviderSnapshot, ContractError> {
    let snapshot: ProviderSnapshot = parse(input, "provider snapshot")?;
    require_wire_version(snapshot.schema_version, "provider snapshot")?;
    require_non_empty(&snapshot.provider_id, "provider id")?;
    if let Some(diagnostic) = &snapshot.diagnostic {
        require_non_empty(&diagnostic.message, "provider diagnostic message")?;
    }
    match snapshot.status {
        ProviderStatus::Online if snapshot.diagnostic.is_some() => Err(ContractError::new(
            "online provider cannot contain a diagnostic",
        )),
        ProviderStatus::Online => Ok(snapshot),
        _ if snapshot.diagnostic.is_none() => {
            Err(ContractError::new("offline provider requires a diagnostic"))
        }
        _ => Ok(snapshot),
    }
}

fn contrast_ratio(foreground: &str, background: &str) -> Result<f64, ContractError> {
    let foreground = relative_luminance(parse_color(foreground)?);
    let background = relative_luminance(parse_color(background)?);
    let (lighter, darker) = if foreground > background {
        (foreground, background)
    } else {
        (background, foreground)
    };
    Ok((lighter + 0.05) / (darker + 0.05))
}

fn parse_color(value: &str) -> Result<[u8; 3], ContractError> {
    if value.len() != 7 || !value.starts_with('#') {
        return Err(ContractError::new("invalid semantic color"));
    }
    let parse = |range| {
        u8::from_str_radix(&value[range], 16)
            .map_err(|_| ContractError::new("invalid semantic color"))
    };
    Ok([parse(1..3)?, parse(3..5)?, parse(5..7)?])
}

fn relative_luminance(color: [u8; 3]) -> f64 {
    let channel = |value: u8| {
        let value = f64::from(value) / 255.0;
        if value <= 0.04045 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * channel(color[0]) + 0.7152 * channel(color[1]) + 0.0722 * channel(color[2])
}

fn require_wire_version(version: u32, name: &str) -> Result<(), ContractError> {
    if version == WIRE_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(ContractError::new(format!(
            "unsupported {name} schema version"
        )))
    }
}

fn require_durable_version(version: u32, name: &str) -> Result<(), ContractError> {
    if version == DURABLE_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(ContractError::new(format!(
            "unsupported {name} schema version"
        )))
    }
}

fn require_non_empty(value: &str, name: &str) -> Result<(), ContractError> {
    if value.is_empty() || value.trim() != value {
        Err(ContractError::new(format!(
            "{name} must be non-empty and unpadded"
        )))
    } else {
        Ok(())
    }
}

fn require_positive(value: u64, name: &str) -> Result<(), ContractError> {
    if value == 0 {
        Err(ContractError::new(format!("{name} must be positive")))
    } else {
        Ok(())
    }
}

fn require_timestamp(value: &str, name: &str) -> Result<(), ContractError> {
    if is_utc_timestamp(value) {
        Ok(())
    } else {
        Err(ContractError::new(format!(
            "{name} must be a UTC timestamp"
        )))
    }
}

fn require_uuid(value: &str, name: &str) -> Result<(), ContractError> {
    let parsed = uuid::Uuid::parse_str(value)
        .map_err(|_| ContractError::new(format!("{name} must be a canonical UUID")))?;
    if parsed.hyphenated().to_string() == value {
        Ok(())
    } else {
        Err(ContractError::new(format!(
            "{name} must be a canonical UUID"
        )))
    }
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
    let n = |a: usize, b: usize| value[a..b].parse::<u32>().ok();
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) =
        (n(0, 4), n(5, 7), n(8, 10), n(11, 13), n(14, 16), n(17, 19))
    else {
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
