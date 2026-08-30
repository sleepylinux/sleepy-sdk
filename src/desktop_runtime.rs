use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{
    validate_calendar_snapshot, validate_desktop_launch_request, validate_notification_document,
    validate_theme_document, validate_weather_snapshot, CalendarSnapshot, CapabilityAvailability,
    CapabilityFailure, ContractError, DesktopLaunchRequest, EventCause, EventCauseKind,
    NotificationDocument, SystemMutation, ThemeDocument, WeatherSnapshot,
};

pub const DESKTOP_WIRE_VERSION: u32 = 3;

const MAX_MONITORS: usize = 64;
const MAX_WORKSPACES: usize = 1_024;
const MAX_WINDOWS: usize = 16_384;
const MAX_ACCESS_POINTS: usize = 4_096;
const MAX_BLUETOOTH_DEVICES: usize = 1_024;
const MAX_AUDIO_NODES: usize = 4_096;
const MAX_AUDIO_STREAMS: usize = 16_384;
const MAX_MEDIA_PLAYERS: usize = 256;
const MAX_TRAY_ITEMS: usize = 1_024;
const MAX_MENU_NODES: usize = 65_536;
const MAX_CLIPBOARD_ENTRIES: usize = 500;
const MAX_ACTIVE_NOTIFICATIONS: usize = 500;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopEnvelope {
    pub schema_version: u32,
    pub generation: u64,
    pub event_id: String,
    pub emitted_at: String,
    pub cause: EventCause,
    pub payload: DesktopEvent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum DesktopEvent {
    FullSnapshot(Box<DesktopSnapshot>),
    CommandResult(DesktopResult),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopSnapshot {
    pub system: DesktopSystemSnapshot,
    pub compositor: DesktopCompositorSnapshot,
    pub notifications: DesktopNotificationSnapshot,
    pub launcher: DesktopLauncherSnapshot,
    pub calendar: CalendarSnapshot,
    pub weather: WeatherSnapshot,
    pub appearance: DesktopAppearanceSnapshot,
    pub resources: DesktopResourceSnapshot,
    pub utilities: DesktopUtilitySnapshot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    rename_all = "camelCase",
    deny_unknown_fields,
    bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>")
)]
pub struct DesktopCapability<T> {
    pub status: CapabilityAvailability,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<CapabilityFailure>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopSystemSnapshot {
    pub network: DesktopCapability<NetworkSnapshot>,
    pub bluetooth: DesktopCapability<BluetoothSnapshot>,
    pub audio: DesktopCapability<AudioSnapshot>,
    pub media: DesktopCapability<MediaSnapshot>,
    pub lock: LockState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkSnapshot {
    pub wifi_enabled: bool,
    pub access_points: Vec<NetworkAccessPoint>,
    pub connections: Vec<NetworkConnection>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkAccessPoint {
    pub id: String,
    pub ssid: String,
    pub signal_level: f64,
    pub secured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkConnection {
    pub id: String,
    pub name: String,
    pub kind: NetworkConnectionKind,
    pub connected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NetworkConnectionKind {
    Ethernet,
    Wifi,
    Vpn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BluetoothSnapshot {
    pub powered: bool,
    pub devices: Vec<BluetoothDevice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BluetoothDevice {
    pub id: String,
    pub name: String,
    pub paired: bool,
    pub connected: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioSnapshot {
    pub nodes: Vec<AudioNode>,
    pub streams: Vec<AudioStream>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioNode {
    pub id: String,
    pub name: String,
    pub kind: AudioNodeKind,
    pub volume: f64,
    pub muted: bool,
    pub is_default: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AudioNodeKind {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioStream {
    pub id: String,
    pub name: String,
    pub node_id: String,
    pub volume: f64,
    pub muted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaSnapshot {
    pub players: Vec<MediaPlayer>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaPlayer {
    pub id: String,
    pub identity: String,
    pub title: String,
    pub artist: String,
    pub playing: bool,
    pub progress: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LockState {
    pub secure: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopCompositorSnapshot {
    pub hyprland: DesktopCapability<HyprlandSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HyprlandSnapshot {
    pub monitors: Vec<Monitor>,
    pub workspaces: Vec<Workspace>,
    pub windows: Vec<Window>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Monitor {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub focused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub monitor_id: String,
    pub focused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Window {
    pub id: String,
    pub title: String,
    pub application_id: String,
    pub workspace_id: String,
    pub focused: bool,
    pub fullscreen: bool,
    pub floating: bool,
    pub pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopNotificationSnapshot {
    pub dnd: bool,
    pub active: Vec<NotificationDocument>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopLauncherSnapshot {
    pub entries: Vec<LauncherEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LauncherEntry {
    pub id: String,
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopAppearanceSnapshot {
    pub theme: ThemeDocument,
    pub wallpaper_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopResourceSnapshot {
    pub samples: Vec<ResourceSample>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResourceSample {
    pub id: String,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub load_one: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopUtilitySnapshot {
    pub tray_items: Vec<TrayItem>,
    pub clipboard_entries: Vec<ClipboardEntry>,
    pub recording: RecordingState,
    pub idle_inhibited: bool,
    pub game_mode: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrayItem {
    pub id: String,
    pub title: String,
    pub menu: TrayMenuNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrayMenuNode {
    pub id: String,
    pub label: String,
    pub enabled: bool,
    pub children: Vec<TrayMenuNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClipboardEntry {
    pub id: String,
    pub preview: String,
    pub mime_type: String,
    pub byte_length: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecordingState {
    pub status: RecordingStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recording_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordingStatus {
    Inactive,
    Recording,
    Paused,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopRequest {
    pub schema_version: u32,
    pub request_id: String,
    pub expected_generation: u64,
    pub command: DesktopCommand,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "family",
    content = "command",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum DesktopCommand {
    System(SystemMutation),
    Compositor(HyprlandCommand),
    Notification(NotificationCommand),
    Launcher(LauncherCommand),
    Appearance(AppearanceCommand),
    Utility(UtilityCommand),
    Session(DesktopSessionCommand),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum HyprlandCommand {
    FocusWindow {
        window_id: String,
    },
    MoveWindowToWorkspace {
        window_id: String,
        workspace_id: String,
    },
    CloseWindow {
        window_id: String,
    },
    FocusWorkspace {
        workspace_id: String,
    },
    MoveWorkspaceToMonitor {
        workspace_id: String,
        monitor_id: String,
    },
    ToggleFullscreen {
        window_id: String,
    },
    ToggleFloating {
        window_id: String,
    },
    TogglePinned {
        window_id: String,
    },
    ToggleGroup {
        window_id: String,
    },
    Exit,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum NotificationCommand {
    SetDnd {
        enabled: bool,
    },
    Archive {
        notification_id: u64,
    },
    InvokeAction {
        notification_id: u64,
        action_id: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum LauncherCommand {
    Launch(DesktopLaunchRequest),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AppearanceCommand {
    ApplyTheme { theme_id: String },
    SetWallpaper { wallpaper_id: String },
    SetReducedMotion { enabled: bool },
    SetOpaque { enabled: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum UtilityCommand {
    InvokeTrayMenu { item_id: String, menu_id: String },
    PasteClipboard { entry_id: String },
    ClearClipboard,
    SetIdleInhibited { enabled: bool },
    StartRecording { output_id: String },
    PauseRecording,
    StopRecording,
    Screenshot { output_id: String },
    PickColor,
    SetGameMode { enabled: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DesktopSessionCommand {
    Lock,
    Suspend,
    Logout,
    Reboot,
    PowerOff,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopResult {
    pub schema_version: u32,
    pub request_id: String,
    pub generation: u64,
    pub status: DesktopResultStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<CapabilityFailure>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DesktopResultStatus {
    Succeeded,
    Failed,
}

pub fn validate_desktop_envelope(input: &str) -> Result<DesktopEnvelope, ContractError> {
    let envelope: DesktopEnvelope = parse(input, "desktop envelope")?;
    require_desktop_version(envelope.schema_version, "desktop envelope")?;
    require_positive(envelope.generation, "desktop envelope generation")?;
    require_canonical_uuid(&envelope.event_id, "desktop eventId")?;
    require_timestamp(&envelope.emitted_at, "desktop emittedAt")?;
    match envelope.cause.kind {
        EventCauseKind::Request => require_canonical_uuid(
            envelope
                .cause
                .request_id
                .as_deref()
                .ok_or_else(|| ContractError::new("request cause requires requestId"))?,
            "desktop cause requestId",
        )?,
        _ if envelope.cause.request_id.is_some() => {
            return Err(ContractError::new(
                "only a request cause may contain requestId",
            ));
        }
        _ => {}
    }
    match &envelope.payload {
        DesktopEvent::FullSnapshot(snapshot) => validate_snapshot(snapshot)?,
        DesktopEvent::CommandResult(result) => {
            validate_result(result)?;
            if result.generation != envelope.generation {
                return Err(ContractError::new(
                    "desktop result generation must match envelope generation",
                ));
            }
            if envelope.cause.kind != EventCauseKind::Request
                || envelope.cause.request_id.as_deref() != Some(result.request_id.as_str())
            {
                return Err(ContractError::new(
                    "desktop result event must name the same requestId",
                ));
            }
        }
    }
    Ok(envelope)
}

pub fn validate_desktop_request(input: &str) -> Result<DesktopRequest, ContractError> {
    let request: DesktopRequest = parse(input, "desktop request")?;
    require_desktop_version(request.schema_version, "desktop request")?;
    require_canonical_uuid(&request.request_id, "desktop requestId")?;
    require_positive(request.expected_generation, "desktop expectedGeneration")?;
    validate_command(&request.command)?;
    Ok(request)
}

pub fn validate_desktop_result(input: &str) -> Result<DesktopResult, ContractError> {
    let result: DesktopResult = parse(input, "desktop result")?;
    validate_result(&result)?;
    Ok(result)
}

fn validate_snapshot(snapshot: &DesktopSnapshot) -> Result<(), ContractError> {
    validate_capability(&snapshot.system.network, "network", |network| {
        validate_unique_ids(
            &network.access_points,
            MAX_ACCESS_POINTS,
            "network access point",
            |item| &item.id,
        )?;
        validate_unique_ids(
            &network.connections,
            usize::MAX,
            "network connection",
            |item| &item.id,
        )?;
        for item in &network.access_points {
            require_non_empty(&item.ssid, "network SSID")?;
            require_normalized(item.signal_level, "network signalLevel")?;
        }
        for item in &network.connections {
            require_non_empty(&item.name, "network connection name")?;
        }
        Ok(())
    })?;
    validate_capability(&snapshot.system.bluetooth, "Bluetooth", |bluetooth| {
        validate_unique_ids(
            &bluetooth.devices,
            MAX_BLUETOOTH_DEVICES,
            "Bluetooth device",
            |item| &item.id,
        )?;
        for item in &bluetooth.devices {
            require_non_empty(&item.name, "Bluetooth device name")?;
        }
        Ok(())
    })?;
    validate_capability(&snapshot.system.audio, "audio", |audio| {
        validate_unique_ids(&audio.nodes, MAX_AUDIO_NODES, "audio node", |item| &item.id)?;
        validate_unique_ids(&audio.streams, MAX_AUDIO_STREAMS, "audio stream", |item| {
            &item.id
        })?;
        for node in &audio.nodes {
            require_non_empty(&node.name, "audio node name")?;
            require_normalized(node.volume, "audio node volume")?;
        }
        for stream in &audio.streams {
            require_non_empty(&stream.name, "audio stream name")?;
            require_non_empty(&stream.node_id, "audio stream nodeId")?;
            require_normalized(stream.volume, "audio stream volume")?;
        }
        Ok(())
    })?;
    validate_capability(&snapshot.system.media, "media", |media| {
        validate_unique_ids(&media.players, MAX_MEDIA_PLAYERS, "media player", |item| {
            &item.id
        })?;
        for player in &media.players {
            require_non_empty(&player.identity, "media player identity")?;
            require_normalized(player.progress, "media player progress")?;
        }
        Ok(())
    })?;

    validate_capability(&snapshot.compositor.hyprland, "Hyprland", |hyprland| {
        validate_unique_ids(&hyprland.monitors, MAX_MONITORS, "monitor", |item| &item.id)?;
        validate_unique_ids(&hyprland.workspaces, MAX_WORKSPACES, "workspace", |item| {
            &item.id
        })?;
        validate_unique_ids(&hyprland.windows, MAX_WINDOWS, "window", |item| &item.id)?;
        for monitor in &hyprland.monitors {
            require_non_empty(&monitor.name, "monitor name")?;
            if monitor.width == 0
                || monitor.height == 0
                || !monitor.scale.is_finite()
                || monitor.scale <= 0.0
            {
                return Err(ContractError::new(
                    "monitor geometry and scale must be positive",
                ));
            }
        }
        for workspace in &hyprland.workspaces {
            require_non_empty(&workspace.name, "workspace name")?;
            require_non_empty(&workspace.monitor_id, "workspace monitorId")?;
        }
        for window in &hyprland.windows {
            require_non_empty(&window.application_id, "window applicationId")?;
            require_non_empty(&window.workspace_id, "window workspaceId")?;
        }
        Ok(())
    })?;

    if snapshot.notifications.active.len() > MAX_ACTIVE_NOTIFICATIONS {
        return Err(ContractError::new(
            "active notifications exceed maximum of 500",
        ));
    }
    let mut notification_ids = BTreeSet::new();
    for notification in &snapshot.notifications.active {
        if !notification_ids.insert(notification.id) {
            return Err(ContractError::new("active notification IDs must be unique"));
        }
        validate_notification_document(&serialize(notification, "notification")?)?;
        let mut action_ids = BTreeSet::new();
        for action in &notification.actions {
            if !action_ids.insert(action.id.as_str()) {
                return Err(ContractError::new("notification action IDs must be unique"));
            }
        }
    }

    validate_unique_ids(
        &snapshot.launcher.entries,
        usize::MAX,
        "launcher entry",
        |item| &item.id,
    )?;
    for entry in &snapshot.launcher.entries {
        require_non_empty(&entry.name, "launcher entry name")?;
        require_non_empty(&entry.icon, "launcher entry icon")?;
    }
    validate_calendar_snapshot(&serialize(&snapshot.calendar, "calendar")?)?;
    validate_unique_ids(
        &snapshot.calendar.events,
        usize::MAX,
        "calendar event",
        |event| &event.id,
    )?;
    validate_weather_snapshot(&serialize(&snapshot.weather, "weather")?)?;
    validate_theme_document(&serialize(&snapshot.appearance.theme, "theme")?)?;
    require_non_empty(&snapshot.appearance.wallpaper_id, "appearance wallpaperId")?;

    validate_unique_ids(
        &snapshot.resources.samples,
        usize::MAX,
        "resource sample",
        |item| &item.id,
    )?;
    for sample in &snapshot.resources.samples {
        require_normalized(sample.cpu_usage, "resource cpuUsage")?;
        require_normalized(sample.memory_usage, "resource memoryUsage")?;
        if !sample.load_one.is_finite() || sample.load_one < 0.0 {
            return Err(ContractError::new(
                "resource loadOne must be finite and non-negative",
            ));
        }
    }

    validate_unique_ids(
        &snapshot.utilities.tray_items,
        MAX_TRAY_ITEMS,
        "tray item",
        |item| &item.id,
    )?;
    let mut menu_ids = BTreeSet::new();
    let mut menu_nodes = 0usize;
    for item in &snapshot.utilities.tray_items {
        require_non_empty(&item.title, "tray item title")?;
        let mut pending = vec![&item.menu];
        while let Some(node) = pending.pop() {
            menu_nodes += 1;
            if menu_nodes > MAX_MENU_NODES {
                return Err(ContractError::new(
                    "tray menu nodes exceed maximum of 65536",
                ));
            }
            require_non_empty(&node.id, "tray menu node id")?;
            require_non_empty(&node.label, "tray menu node label")?;
            if !menu_ids.insert(node.id.as_str()) {
                return Err(ContractError::new("tray menu node IDs must be unique"));
            }
            pending.extend(&node.children);
        }
    }
    validate_unique_ids(
        &snapshot.utilities.clipboard_entries,
        MAX_CLIPBOARD_ENTRIES,
        "clipboard entry",
        |item| &item.id,
    )?;
    for entry in &snapshot.utilities.clipboard_entries {
        require_non_empty(&entry.mime_type, "clipboard entry mimeType")?;
    }
    validate_recording_state(&snapshot.utilities.recording)?;
    Ok(())
}

fn validate_capability<T>(
    capability: &DesktopCapability<T>,
    name: &str,
    validate_data: impl FnOnce(&T) -> Result<(), ContractError>,
) -> Result<(), ContractError> {
    match capability.status {
        CapabilityAvailability::Available => {
            if capability.diagnostic.is_some() {
                return Err(ContractError::new(format!(
                    "available {name} capability cannot contain a diagnostic"
                )));
            }
            let data = capability.data.as_ref().ok_or_else(|| {
                ContractError::new(format!("available {name} capability requires data"))
            })?;
            validate_data(data)
        }
        _ => {
            if capability.data.is_some() || capability.diagnostic.is_none() {
                return Err(ContractError::new(format!(
                    "unavailable {name} capability requires only a diagnostic"
                )));
            }
            require_non_empty(
                &capability
                    .diagnostic
                    .as_ref()
                    .expect("checked above")
                    .message,
                &format!("{name} diagnostic message"),
            )
        }
    }
}

fn validate_recording_state(recording: &RecordingState) -> Result<(), ContractError> {
    match recording.status {
        RecordingStatus::Inactive
            if recording.recording_id.is_some() || recording.output_id.is_some() =>
        {
            Err(ContractError::new(
                "inactive recording cannot contain recording or output IDs",
            ))
        }
        RecordingStatus::Recording | RecordingStatus::Paused => {
            require_non_empty(
                recording.recording_id.as_deref().unwrap_or_default(),
                "recording id",
            )?;
            require_non_empty(
                recording.output_id.as_deref().unwrap_or_default(),
                "recording outputId",
            )
        }
        RecordingStatus::Inactive => Ok(()),
    }
}

fn validate_command(command: &DesktopCommand) -> Result<(), ContractError> {
    match command {
        DesktopCommand::System(_) => Ok(()),
        DesktopCommand::Compositor(command) => validate_hyprland_command(command),
        DesktopCommand::Notification(command) => match command {
            NotificationCommand::SetDnd { .. } => Ok(()),
            NotificationCommand::Archive { notification_id }
            | NotificationCommand::InvokeAction {
                notification_id, ..
            } => {
                require_positive(*notification_id, "notification command id")?;
                if let NotificationCommand::InvokeAction { action_id, .. } = command {
                    require_non_empty(action_id, "notification actionId")?;
                }
                Ok(())
            }
        },
        DesktopCommand::Launcher(LauncherCommand::Launch(request)) => {
            validate_desktop_launch_request(&serialize(request, "desktop launch request")?)?;
            Ok(())
        }
        DesktopCommand::Appearance(command) => match command {
            AppearanceCommand::ApplyTheme { theme_id } => {
                require_non_empty(theme_id, "appearance themeId")
            }
            AppearanceCommand::SetWallpaper { wallpaper_id } => {
                require_non_empty(wallpaper_id, "appearance wallpaperId")
            }
            AppearanceCommand::SetReducedMotion { .. } | AppearanceCommand::SetOpaque { .. } => {
                Ok(())
            }
        },
        DesktopCommand::Utility(command) => match command {
            UtilityCommand::InvokeTrayMenu { item_id, menu_id } => {
                require_non_empty(item_id, "tray itemId")?;
                require_non_empty(menu_id, "tray menuId")
            }
            UtilityCommand::PasteClipboard { entry_id } => {
                require_non_empty(entry_id, "clipboard entryId")
            }
            UtilityCommand::StartRecording { output_id }
            | UtilityCommand::Screenshot { output_id } => {
                require_non_empty(output_id, "utility outputId")
            }
            UtilityCommand::ClearClipboard
            | UtilityCommand::SetIdleInhibited { .. }
            | UtilityCommand::PauseRecording
            | UtilityCommand::StopRecording
            | UtilityCommand::PickColor
            | UtilityCommand::SetGameMode { .. } => Ok(()),
        },
        DesktopCommand::Session(_) => Ok(()),
    }
}

fn validate_hyprland_command(command: &HyprlandCommand) -> Result<(), ContractError> {
    match command {
        HyprlandCommand::FocusWindow { window_id }
        | HyprlandCommand::CloseWindow { window_id }
        | HyprlandCommand::ToggleFullscreen { window_id }
        | HyprlandCommand::ToggleFloating { window_id }
        | HyprlandCommand::TogglePinned { window_id }
        | HyprlandCommand::ToggleGroup { window_id } => {
            require_non_empty(window_id, "Hyprland windowId")
        }
        HyprlandCommand::MoveWindowToWorkspace {
            window_id,
            workspace_id,
        } => {
            require_non_empty(window_id, "Hyprland windowId")?;
            require_non_empty(workspace_id, "Hyprland workspaceId")
        }
        HyprlandCommand::FocusWorkspace { workspace_id } => {
            require_non_empty(workspace_id, "Hyprland workspaceId")
        }
        HyprlandCommand::MoveWorkspaceToMonitor {
            workspace_id,
            monitor_id,
        } => {
            require_non_empty(workspace_id, "Hyprland workspaceId")?;
            require_non_empty(monitor_id, "Hyprland monitorId")
        }
        HyprlandCommand::Exit => Ok(()),
    }
}

fn validate_result(result: &DesktopResult) -> Result<(), ContractError> {
    require_desktop_version(result.schema_version, "desktop result")?;
    require_canonical_uuid(&result.request_id, "desktop result requestId")?;
    require_positive(result.generation, "desktop result generation")?;
    match result.status {
        DesktopResultStatus::Succeeded if result.diagnostic.is_some() => Err(ContractError::new(
            "successful desktop result cannot contain a diagnostic",
        )),
        DesktopResultStatus::Failed if result.diagnostic.is_none() => Err(ContractError::new(
            "failed desktop result requires a diagnostic",
        )),
        _ => Ok(()),
    }
}

fn validate_unique_ids<T>(
    items: &[T],
    maximum: usize,
    name: &str,
    id: impl Fn(&T) -> &str,
) -> Result<(), ContractError> {
    if items.len() > maximum {
        return Err(ContractError::new(format!(
            "{name} collection exceeds maximum of {maximum}"
        )));
    }
    let mut identifiers = BTreeSet::new();
    for item in items {
        let identifier = id(item);
        require_non_empty(identifier, &format!("{name} id"))?;
        if !identifiers.insert(identifier) {
            return Err(ContractError::new(format!("{name} IDs must be unique")));
        }
    }
    Ok(())
}

fn require_desktop_version(version: u32, name: &str) -> Result<(), ContractError> {
    if version == DESKTOP_WIRE_VERSION {
        Ok(())
    } else {
        Err(ContractError::new(format!(
            "{name} schemaVersion must be {DESKTOP_WIRE_VERSION}"
        )))
    }
}

fn require_positive(value: u64, name: &str) -> Result<(), ContractError> {
    if value == 0 {
        Err(ContractError::new(format!("{name} must be positive")))
    } else {
        Ok(())
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

fn require_normalized(value: f64, name: &str) -> Result<(), ContractError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(ContractError::new(format!(
            "{name} must be finite and normalized"
        )))
    }
}

fn require_canonical_uuid(value: &str, name: &str) -> Result<(), ContractError> {
    let canonical = uuid::Uuid::parse_str(value)
        .map(|identifier| identifier.hyphenated().to_string())
        .unwrap_or_default();
    if canonical == value {
        Ok(())
    } else {
        Err(ContractError::new(format!(
            "{name} must be a canonical hyphenated UUID"
        )))
    }
}

fn require_timestamp(value: &str, name: &str) -> Result<(), ContractError> {
    if value.contains('T') && value.ends_with('Z') {
        Ok(())
    } else {
        Err(ContractError::new(format!(
            "{name} must be a UTC timestamp"
        )))
    }
}

fn parse<T: for<'de> Deserialize<'de>>(input: &str, name: &str) -> Result<T, ContractError> {
    serde_json::from_str(input)
        .map_err(|error| ContractError::new(format!("invalid {name}: {error}")))
}

fn serialize<T: Serialize>(value: &T, name: &str) -> Result<String, ContractError> {
    serde_json::to_string(value)
        .map_err(|error| ContractError::new(format!("invalid {name}: {error}")))
}
