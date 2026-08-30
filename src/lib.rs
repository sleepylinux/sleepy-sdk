//! Versioned public document contracts for Sleepy Linux.

mod desktop;
mod desktop_runtime;
mod domains;
mod events;
mod keybindings;
mod system;

pub use events::{
    validate_event_envelope, validate_mutation_exchange, validate_mutation_request,
    validate_mutation_result, AudioRuntimeState, BatteryRuntimeState, BluetoothRuntimeState,
    BrightnessRuntimeState, CapabilityAvailability, CapabilityFailure, CapabilityRecord,
    CapabilityValue, Connectivity, DaemonCommand, EventCause, EventCauseKind, EventEnvelope,
    LifecycleEvent, LifecycleState, MediaRuntimeState, MutationFailure, MutationRequest,
    MutationResult, MutationStatus, NetworkRuntimeState, NightLightRuntimeState, NiriEvent,
    NiriRuntimeState, NotificationChange, NotificationEvent, PowerProfileRuntimeState,
    ProviderEvent, ResourceRuntimeState, RuntimeCapabilityId, RuntimeSnapshot, SessionEvent,
    ThemeEvent, WIRE_SCHEMA_VERSION,
};

pub use keybindings::{
    canonicalize_accelerator, packaged_reserved_keybindings, validate_keybindings,
    validate_keybindings_with_reserved, ConflictKind, KeybindingConflict, SemanticAction,
    KNOWN_SEMANTIC_ACTIONS,
};
pub use system::{
    validate_session_action_request, validate_session_action_result,
    validate_system_mutation_result, validate_system_snapshot, AudioOutputDevice, AudioState,
    BluetoothState, CapabilityDiagnostic, CapabilityErrorKind, CapabilityId, CapabilityState,
    DisplayState, MediaState, MediaTransport, NetworkState, PowerProfile, PowerState,
    SessionAction, SessionActionRequest, SessionActionResult, SessionActionStatus, SystemMutation,
    SystemMutationResult, SystemSnapshot,
};

use std::{
    collections::BTreeMap,
    error::Error,
    fmt,
    path::{Component, Path},
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const CONTRACT_SCHEMA_VERSION: u32 = 1;
pub const PLUGIN_API_VERSION: u32 = 1;
pub const BUILTIN_PRESET_ID: &str = "builtin.sleepy";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractError(String);

impl ContractError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Error for ContractError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SettingsDocument {
    pub schema_version: u32,
    pub active_preset_id: String,
    pub appearance_mode: AppearanceMode,
    pub palette_source: PaletteSource,
    pub reduced_motion: bool,
    pub effects_profile: EffectsProfile,
    pub panel_visibility: PanelVisibility,
    pub web_search_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppearanceMode {
    Dark,
    Light,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaletteSource {
    Sleepy,
    System,
    Custom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EffectsProfile {
    Full,
    Reduced,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PanelVisibility {
    Always,
    AutoHide,
    Hidden,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresetDocument {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub origin: PresetOrigin,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_preset_id: Option<String>,
    pub layouts: BTreeMap<String, serde_json::Value>,
    pub drawers: BTreeMap<String, serde_json::Value>,
    pub keybindings: BTreeMap<String, String>,
    pub plugin_requirements: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PresetOrigin {
    Builtin,
    User,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginManifest {
    pub schema_version: u32,
    pub id: String,
    pub version: String,
    pub api_version: u32,
    pub entrypoint: String,
    pub surface_kinds: Vec<SurfaceKind>,
    pub capabilities: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings_schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SurfaceKind {
    Rail,
    Drawer,
}

pub fn validate_settings(input: &str) -> Result<SettingsDocument, ContractError> {
    let document: SettingsDocument = parse_document(input)?;
    validate_schema_version(document.schema_version, "settings")?;
    require_non_empty(&document.active_preset_id, "settings activePresetId")?;
    Ok(document)
}

pub fn validate_preset(input: &str) -> Result<PresetDocument, ContractError> {
    let document: PresetDocument = parse_document(input)?;
    validate_schema_version(document.schema_version, "preset")?;
    require_non_empty(&document.name, "preset name")?;

    match document.origin {
        PresetOrigin::Builtin if document.id != BUILTIN_PRESET_ID => {
            return Err(ContractError::new(format!(
                "builtin preset id must be {BUILTIN_PRESET_ID}"
            )));
        }
        PresetOrigin::User
            if uuid::Uuid::parse_str(&document.id)
                .map(|identifier| {
                    identifier.hyphenated().to_string() != document.id.to_ascii_lowercase()
                })
                .unwrap_or(true) =>
        {
            return Err(ContractError::new(
                "user preset id must be a canonical hyphenated UUID",
            ));
        }
        _ => {}
    }

    if let Some(base_preset_id) = &document.base_preset_id {
        require_non_empty(base_preset_id, "preset basePresetId")?;
    }

    for display_id in document.layouts.keys() {
        require_non_empty(display_id, "preset display identity")?;
    }
    for drawer_id in document.drawers.keys() {
        require_non_empty(drawer_id, "preset drawer id")?;
    }
    validate_keybindings(&document.keybindings)?;
    for plugin_id in &document.plugin_requirements {
        require_non_empty(plugin_id, "preset plugin requirement")?;
    }

    Ok(document)
}

pub fn validate_plugin_manifest(input: &str) -> Result<PluginManifest, ContractError> {
    let document: PluginManifest = parse_document(input)?;
    validate_schema_version(document.schema_version, "plugin manifest")?;
    if document.api_version != PLUGIN_API_VERSION {
        return Err(ContractError::new(format!(
            "plugin apiVersion must be {PLUGIN_API_VERSION}"
        )));
    }
    require_non_empty(&document.id, "plugin id")?;
    semver::Version::parse(&document.version)
        .map_err(|error| ContractError::new(format!("plugin version must be semantic: {error}")))?;
    validate_entrypoint(&document.entrypoint)?;
    for capability in &document.capabilities {
        require_non_empty(capability, "plugin capability")?;
    }
    if let Some(settings_schema) = &document.settings_schema {
        if !settings_schema.is_object() {
            return Err(ContractError::new(
                "plugin settingsSchema must be an object",
            ));
        }
    }

    Ok(document)
}

fn parse_document<T: DeserializeOwned>(input: &str) -> Result<T, ContractError> {
    serde_json::from_str(input).map_err(|error| ContractError::new(error.to_string()))
}

fn validate_schema_version(version: u32, document_name: &str) -> Result<(), ContractError> {
    if version == CONTRACT_SCHEMA_VERSION {
        Ok(())
    } else {
        Err(ContractError::new(format!(
            "{document_name} schemaVersion must be {CONTRACT_SCHEMA_VERSION}"
        )))
    }
}

fn require_non_empty(value: &str, field_name: &str) -> Result<(), ContractError> {
    if value.trim().is_empty() {
        Err(ContractError::new(format!(
            "{field_name} must not be empty"
        )))
    } else {
        Ok(())
    }
}

fn validate_entrypoint(entrypoint: &str) -> Result<(), ContractError> {
    if entrypoint.is_empty() || entrypoint.trim() != entrypoint {
        return Err(ContractError::new(
            "plugin entrypoint must not be empty or padded",
        ));
    }
    if entrypoint.contains('\\') || !entrypoint.ends_with(".qml") {
        return Err(ContractError::new(
            "plugin entrypoint must be a slash-separated .qml file",
        ));
    }

    for component in Path::new(entrypoint).components() {
        if !matches!(component, Component::Normal(_)) {
            return Err(ContractError::new(
                "plugin entrypoint must be relative to its package without traversal",
            ));
        }
    }

    Ok(())
}
pub use desktop::{
    validate_calendar_snapshot, validate_desktop_launch_request, validate_osd_event,
    validate_weather_snapshot, CalendarEvent, CalendarProvider, CalendarSnapshot,
    CalendarSourceError, DesktopLaunchRequest, ForecastPoint, OsdEvent, OsdKind, WeatherLocation,
    WeatherProvider, WeatherSnapshot,
};
pub use desktop_runtime::{
    validate_desktop_envelope, validate_desktop_request, validate_desktop_result,
    AppearanceCommand, AudioNode, AudioNodeKind, AudioSnapshot, AudioStream, BluetoothDevice,
    BluetoothSnapshot, ClipboardEntry, DesktopAppearanceSnapshot, DesktopCapability,
    DesktopCommand, DesktopCompositorSnapshot, DesktopEnvelope, DesktopEvent,
    DesktopLauncherSnapshot, DesktopNotificationSnapshot, DesktopRequest, DesktopResourceSnapshot,
    DesktopResult, DesktopResultStatus, DesktopSessionCommand, DesktopSnapshot,
    DesktopSystemSnapshot, DesktopUtilitySnapshot, HyprlandCommand, HyprlandSnapshot,
    LauncherCommand, LauncherEntry, LockState, MediaPlayer, MediaSnapshot, Monitor,
    NetworkAccessPoint, NetworkConnection, NetworkConnectionKind, NetworkSnapshot,
    NotificationCommand, RecordingState, RecordingStatus, ResourceSample, TrayItem, TrayMenuNode,
    UtilityCommand, Window, Workspace, DESKTOP_WIRE_VERSION,
};
pub use domains::{
    validate_hardware_capability_snapshot, validate_installation_profile,
    validate_notification_document, validate_provider_snapshot, validate_theme_document,
    CacheStatus, DeviceIdentifier, DeviceKind, FirstBootSnapshot, FirstBootState, FixtureMetadata,
    HardwareCapability, HardwareCapabilitySnapshot, HardwareDeviceCapability, InstallationProfile,
    InstallerProvider, MachineProfile, MachineSystem, NotificationAction, NotificationActionState,
    NotificationDocument, NotificationUrgency, ProviderKind, ProviderSnapshot, ProviderStatus,
    RollbackSnapshot, SemanticColors, ThemeAppearance, ThemeDocument, ThemeEffects, ThemeOrigin,
    DURABLE_SCHEMA_VERSION,
};
