use std::{collections::BTreeSet, fmt, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::{
    validate_calendar_snapshot, validate_desktop_launch_request, validate_notification_document,
    validate_osd_event, validate_theme_document, validate_weather_snapshot, CalendarSnapshot,
    CapabilityAvailability, CapabilityFailure, ContractError, DesktopLaunchRequest, EventCause,
    EventCauseKind, MediaTransport, NotificationDocument, OsdEvent, PowerProfile, SystemMutation,
    ThemeDocument, WeatherSnapshot,
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
const MAX_OSD_HISTORY: usize = 500;

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
    DomainUpdate(DesktopDomainUpdate),
    CommandResult(DesktopResult),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "topic",
    content = "update",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum DesktopDomainUpdate {
    System(DesktopSystemUpdate),
    Compositor(DesktopCompositorUpdate),
    Notifications(DesktopNotificationSnapshot),
    Launcher(DesktopLauncherSnapshot),
    Calendar(DesktopCalendarSnapshot),
    Weather(DesktopWeatherSnapshot),
    Appearance(DesktopAppearanceSnapshot),
    Resources(DesktopResourceSnapshot),
    Utilities(DesktopUtilityUpdate),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "domain",
    content = "data",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum DesktopSystemUpdate {
    Network(DesktopCapability<NetworkSnapshot>),
    Bluetooth(DesktopCapability<BluetoothSnapshot>),
    Audio(DesktopCapability<AudioSnapshot>),
    Media(DesktopCapability<MediaSnapshot>),
    Battery(DesktopCapability<BatterySnapshot>),
    Brightness(DesktopCapability<BrightnessSnapshot>),
    NightLight(DesktopCapability<NightLightSnapshot>),
    Power(DesktopCapability<DesktopPowerSnapshot>),
    Osd(DesktopCapability<DesktopOsdSnapshot>),
    Lock(DesktopCapability<LockState>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "domain",
    content = "data",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum DesktopCompositorUpdate {
    Hyprland(DesktopCapability<HyprlandSnapshot>),
    Monitors(Vec<Monitor>),
    Workspaces(Vec<Workspace>),
    Windows(Vec<Window>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopSnapshot {
    pub system: DesktopSystemSnapshot,
    pub compositor: DesktopCompositorSnapshot,
    pub notifications: DesktopNotificationSnapshot,
    pub launcher: DesktopLauncherSnapshot,
    pub calendar: DesktopCalendarSnapshot,
    pub weather: DesktopWeatherSnapshot,
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
    #[serde(
        default,
        deserialize_with = "deserialize_optional_non_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub data: Option<T>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_non_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub diagnostic: Option<CapabilityFailure>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProducerAvailability {
    pub status: CapabilityAvailability,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_non_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub diagnostic: Option<CapabilityFailure>,
}

fn deserialize_optional_non_null<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct NonNullOptionVisitor<T>(PhantomData<T>);

    impl<'de, T> serde::de::Visitor<'de> for NonNullOptionVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Option<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a non-null value")
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            T::deserialize(deserializer).map(Some)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Err(E::custom("explicit null is not allowed"))
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Err(E::custom("explicit null is not allowed"))
        }
    }

    deserializer.deserialize_option(NonNullOptionVisitor(PhantomData))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopSystemSnapshot {
    pub network: DesktopCapability<NetworkSnapshot>,
    pub bluetooth: DesktopCapability<BluetoothSnapshot>,
    pub audio: DesktopCapability<AudioSnapshot>,
    pub media: DesktopCapability<MediaSnapshot>,
    pub battery: DesktopCapability<BatterySnapshot>,
    pub brightness: DesktopCapability<BrightnessSnapshot>,
    pub night_light: DesktopCapability<NightLightSnapshot>,
    pub power: DesktopCapability<DesktopPowerSnapshot>,
    pub osd: DesktopCapability<DesktopOsdSnapshot>,
    pub lock: DesktopCapability<LockState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NetworkSnapshot {
    pub wifi_enabled: bool,
    pub scanning: bool,
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
    pub scanning: bool,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BatterySnapshot {
    pub level: f64,
    pub charging: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seconds_remaining: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BrightnessSnapshot {
    pub level: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NightLightSnapshot {
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopPowerSnapshot {
    pub active_profile: PowerProfile,
    pub available_profiles: Vec<PowerProfile>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopOsdSnapshot {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<OsdEvent>,
    pub history: Vec<OsdEvent>,
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
    pub action_capabilities: HyprlandActionCapabilities,
    pub monitors: Vec<Monitor>,
    pub workspaces: Vec<Workspace>,
    pub windows: Vec<Window>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HyprlandActionCapabilities {
    pub focus_window: bool,
    pub move_window_to_workspace: bool,
    pub close_window: bool,
    pub focus_workspace: bool,
    pub move_workspace_to_monitor: bool,
    pub toggle_fullscreen: bool,
    pub toggle_floating: bool,
    pub toggle_pinned: bool,
    pub toggle_group: bool,
    pub exit: bool,
}

impl HyprlandActionCapabilities {
    pub fn supports(&self, command: &HyprlandCommand) -> bool {
        match command {
            HyprlandCommand::FocusWindow { .. } => self.focus_window,
            HyprlandCommand::MoveWindowToWorkspace { .. } => self.move_window_to_workspace,
            HyprlandCommand::CloseWindow { .. } => self.close_window,
            HyprlandCommand::FocusWorkspace { .. } => self.focus_workspace,
            HyprlandCommand::MoveWorkspaceToMonitor { .. } => self.move_workspace_to_monitor,
            HyprlandCommand::ToggleFullscreen { .. } => self.toggle_fullscreen,
            HyprlandCommand::ToggleFloating { .. } => self.toggle_floating,
            HyprlandCommand::TogglePinned { .. } => self.toggle_pinned,
            HyprlandCommand::ToggleGroup { .. } => self.toggle_group,
            HyprlandCommand::Exit => self.exit,
        }
    }
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
    pub grouped: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopNotificationSnapshot {
    pub availability: ProducerAvailability,
    pub dnd: bool,
    pub active: Vec<NotificationDocument>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopLauncherSnapshot {
    pub availability: ProducerAvailability,
    pub entries: Vec<LauncherEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopCalendarSnapshot {
    pub availability: ProducerAvailability,
    pub snapshot: CalendarSnapshot,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopWeatherSnapshot {
    pub availability: ProducerAvailability,
    pub snapshot: WeatherSnapshot,
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
    pub availability: ProducerAvailability,
    pub theme: ThemeDocument,
    pub wallpaper_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopResourceSnapshot {
    pub availability: ProducerAvailability,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopUtilitySnapshot {
    pub tray_items: DesktopCapability<Vec<TrayItem>>,
    pub clipboard_entries: DesktopCapability<Vec<ClipboardEntry>>,
    pub recording: DesktopCapability<RecordingState>,
    pub idle_inhibited: DesktopCapability<bool>,
    pub game_mode: DesktopCapability<bool>,
    pub screenshot: ProducerAvailability,
    pub color_picker: ProducerAvailability,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "domain",
    content = "data",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum DesktopUtilityUpdate {
    TrayItems(DesktopCapability<Vec<TrayItem>>),
    ClipboardEntries(DesktopCapability<Vec<ClipboardEntry>>),
    Recording(DesktopCapability<RecordingState>),
    IdleInhibited(DesktopCapability<bool>),
    GameMode(DesktopCapability<bool>),
    Screenshot(ProducerAvailability),
    ColorPicker(ProducerAvailability),
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StableId(pub String);

impl StableId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "family",
    content = "command",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum DesktopCommand {
    System(DesktopSystemCommand),
    Compositor(HyprlandCommand),
    Notification(NotificationCommand),
    Launcher(LauncherCommand),
    Appearance(AppearanceCommand),
    Utility(UtilityCommand),
    Session(DesktopSessionCommand),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DesktopSystemCommand {
    Legacy(SystemMutation),
    Domain(DesktopSystemMutation),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "domain",
    content = "action",
    rename_all = "camelCase",
    deny_unknown_fields
)]
pub enum DesktopSystemMutation {
    Network(NetworkCommand),
    Bluetooth(BluetoothCommand),
    Audio(AudioCommand),
    Media(MediaCommand),
    Display(DisplayCommand),
    Power(PowerCommand),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum NetworkCommand {
    SetWifiEnabled { enabled: bool },
    ScanWifi,
    ConnectWifi { access_point_id: StableId },
    Disconnect { connection_id: StableId },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum BluetoothCommand {
    SetPowered { powered: bool },
    Scan,
    Pair { device_id: StableId },
    Connect { device_id: StableId },
    Disconnect { device_id: StableId },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum AudioCommand {
    SetDefaultNode { node_id: StableId },
    SetNodeVolume { node_id: StableId, level: f64 },
    SetNodeMuted { node_id: StableId, muted: bool },
    SetStreamVolume { stream_id: StableId, level: f64 },
    SetStreamMuted { stream_id: StableId, muted: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum MediaCommand {
    Transport {
        player_id: StableId,
        transport: MediaTransport,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum DisplayCommand {
    SetBrightness { output_id: StableId, level: f64 },
    SetNightLightEnabled { enabled: bool },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "data",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum PowerCommand {
    SetProfile { profile: PowerProfile },
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
        window_id: StableId,
    },
    MoveWindowToWorkspace {
        window_id: StableId,
        workspace_id: StableId,
    },
    CloseWindow {
        window_id: StableId,
    },
    FocusWorkspace {
        workspace_id: StableId,
    },
    MoveWorkspaceToMonitor {
        workspace_id: StableId,
        monitor_id: StableId,
    },
    ToggleFullscreen {
        window_id: StableId,
    },
    ToggleFloating {
        window_id: StableId,
    },
    TogglePinned {
        window_id: StableId,
    },
    ToggleGroup {
        window_id: StableId,
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
        action_id: StableId,
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
    ApplyTheme { theme_id: StableId },
    SetWallpaper { wallpaper_id: StableId },
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
    InvokeTrayMenu {
        item_id: StableId,
        menu_id: StableId,
    },
    PasteClipboard {
        entry_id: StableId,
    },
    ClearClipboard,
    SetIdleInhibited {
        enabled: bool,
    },
    StartRecording {
        output_id: StableId,
        #[serde(default)]
        target: RecordingTarget,
        #[serde(default)]
        audio: bool,
    },
    PauseRecording,
    StopRecording,
    DeleteRecording {
        recording_id: StableId,
    },
    Screenshot {
        output_id: StableId,
    },
    PickColor,
    SetGameMode {
        enabled: bool,
    },
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RecordingTarget {
    #[default]
    Output,
    Region,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DesktopSessionCommand {
    Lock,
    Suspend,
    Hibernate,
    SuspendThenHibernate,
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
    #[serde(
        default,
        deserialize_with = "deserialize_optional_non_null",
        skip_serializing_if = "Option::is_none"
    )]
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
        DesktopEvent::DomainUpdate(update) => validate_domain_update(update)?,
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
            if !audio.nodes.iter().any(|node| node.id == stream.node_id) {
                return Err(ContractError::new(
                    "audio stream nodeId must reference a present audio node",
                ));
            }
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
    validate_capability(&snapshot.system.battery, "battery", |battery| {
        require_normalized(battery.level, "battery level")
    })?;
    validate_capability(&snapshot.system.brightness, "brightness", |brightness| {
        require_normalized(brightness.level, "brightness level")
    })?;
    validate_capability(&snapshot.system.night_light, "night light", |_| Ok(()))?;
    validate_capability(&snapshot.system.power, "power", |power| {
        if power.available_profiles.is_empty() {
            return Err(ContractError::new(
                "power availableProfiles must not be empty",
            ));
        }
        let profiles: BTreeSet<_> = power.available_profiles.iter().collect();
        if profiles.len() != power.available_profiles.len()
            || !profiles.contains(&power.active_profile)
        {
            return Err(ContractError::new(
                "power profiles must be unique and contain activeProfile",
            ));
        }
        Ok(())
    })?;
    validate_capability(&snapshot.system.osd, "OSD", validate_osd_snapshot)?;
    validate_capability(&snapshot.system.lock, "lock", |_| Ok(()))?;

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
            if !hyprland
                .monitors
                .iter()
                .any(|monitor| monitor.id == workspace.monitor_id)
            {
                return Err(ContractError::new(
                    "workspace monitorId must reference a present monitor",
                ));
            }
        }
        for window in &hyprland.windows {
            require_non_empty(&window.application_id, "window applicationId")?;
            require_non_empty(&window.workspace_id, "window workspaceId")?;
            if !hyprland
                .workspaces
                .iter()
                .any(|workspace| workspace.id == window.workspace_id)
            {
                return Err(ContractError::new(
                    "window workspaceId must reference a present workspace",
                ));
            }
        }
        if hyprland
            .monitors
            .iter()
            .filter(|monitor| monitor.focused)
            .count()
            > 1
        {
            return Err(ContractError::new(
                "Hyprland snapshot may contain at most one focused monitor",
            ));
        }
        if hyprland
            .workspaces
            .iter()
            .filter(|workspace| workspace.focused)
            .count()
            > 1
        {
            return Err(ContractError::new(
                "Hyprland snapshot may contain at most one focused workspace",
            ));
        }
        if hyprland
            .windows
            .iter()
            .filter(|window| window.focused)
            .count()
            > 1
        {
            return Err(ContractError::new(
                "Hyprland snapshot may contain at most one focused window",
            ));
        }
        Ok(())
    })?;

    validate_producer_availability(&snapshot.notifications.availability, "notifications")?;
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
        validate_notification_v3(notification)?;
    }

    validate_producer_availability(&snapshot.launcher.availability, "launcher")?;
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
    validate_producer_availability(&snapshot.calendar.availability, "calendar")?;
    validate_calendar_v3(&snapshot.calendar.snapshot)?;
    validate_producer_availability(&snapshot.weather.availability, "weather")?;
    validate_weather_v3(&snapshot.weather.snapshot)?;
    validate_producer_availability(&snapshot.appearance.availability, "appearance")?;
    validate_theme_document(&serialize(&snapshot.appearance.theme, "theme")?)?;
    require_non_empty(&snapshot.appearance.wallpaper_id, "appearance wallpaperId")?;

    validate_producer_availability(&snapshot.resources.availability, "resources")?;
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

    validate_utilities_snapshot(&snapshot.utilities)
}

fn validate_domain_update(update: &DesktopDomainUpdate) -> Result<(), ContractError> {
    match update {
        DesktopDomainUpdate::System(update) => validate_system_update(update),
        DesktopDomainUpdate::Compositor(update) => validate_compositor_update(update),
        DesktopDomainUpdate::Notifications(snapshot) => {
            validate_producer_availability(&snapshot.availability, "notifications")?;
            if snapshot.active.len() > MAX_ACTIVE_NOTIFICATIONS {
                return Err(ContractError::new(
                    "active notifications exceed maximum of 500",
                ));
            }
            let mut ids = BTreeSet::new();
            for notification in &snapshot.active {
                if !ids.insert(notification.id) {
                    return Err(ContractError::new("active notification IDs must be unique"));
                }
                validate_notification_v3(notification)?;
            }
            Ok(())
        }
        DesktopDomainUpdate::Launcher(snapshot) => {
            validate_producer_availability(&snapshot.availability, "launcher")?;
            validate_unique_ids(&snapshot.entries, usize::MAX, "launcher entry", |entry| {
                &entry.id
            })?;
            for entry in &snapshot.entries {
                require_non_empty(&entry.name, "launcher entry name")?;
                require_non_empty(&entry.icon, "launcher entry icon")?;
            }
            Ok(())
        }
        DesktopDomainUpdate::Calendar(snapshot) => {
            validate_producer_availability(&snapshot.availability, "calendar")?;
            validate_calendar_v3(&snapshot.snapshot)
        }
        DesktopDomainUpdate::Weather(snapshot) => {
            validate_producer_availability(&snapshot.availability, "weather")?;
            validate_weather_v3(&snapshot.snapshot)
        }
        DesktopDomainUpdate::Appearance(snapshot) => {
            validate_producer_availability(&snapshot.availability, "appearance")?;
            validate_theme_document(&serialize(&snapshot.theme, "theme")?)?;
            require_non_empty(&snapshot.wallpaper_id, "appearance wallpaperId")
        }
        DesktopDomainUpdate::Resources(snapshot) => {
            validate_producer_availability(&snapshot.availability, "resources")?;
            validate_unique_ids(&snapshot.samples, usize::MAX, "resource sample", |sample| {
                &sample.id
            })?;
            for sample in &snapshot.samples {
                require_normalized(sample.cpu_usage, "resource cpuUsage")?;
                require_normalized(sample.memory_usage, "resource memoryUsage")?;
                if !sample.load_one.is_finite() || sample.load_one < 0.0 {
                    return Err(ContractError::new(
                        "resource loadOne must be finite and non-negative",
                    ));
                }
            }
            Ok(())
        }
        DesktopDomainUpdate::Utilities(update) => validate_utility_update(update),
    }
}

fn validate_system_update(update: &DesktopSystemUpdate) -> Result<(), ContractError> {
    match update {
        DesktopSystemUpdate::Network(capability) => {
            validate_capability(capability, "network", |network| {
                validate_unique_ids(
                    &network.access_points,
                    MAX_ACCESS_POINTS,
                    "network access point",
                    |item| &item.id,
                )?;
                for access_point in &network.access_points {
                    require_non_empty(&access_point.ssid, "network SSID")?;
                    require_normalized(access_point.signal_level, "network signalLevel")?;
                }
                validate_unique_ids(
                    &network.connections,
                    usize::MAX,
                    "network connection",
                    |item| &item.id,
                )?;
                for connection in &network.connections {
                    require_non_empty(&connection.name, "network connection name")?;
                }
                Ok(())
            })
        }
        DesktopSystemUpdate::Bluetooth(capability) => {
            validate_capability(capability, "Bluetooth", |bluetooth| {
                validate_unique_ids(
                    &bluetooth.devices,
                    MAX_BLUETOOTH_DEVICES,
                    "Bluetooth device",
                    |item| &item.id,
                )?;
                for device in &bluetooth.devices {
                    require_non_empty(&device.name, "Bluetooth device name")?;
                }
                Ok(())
            })
        }
        DesktopSystemUpdate::Audio(capability) => {
            validate_capability(capability, "audio", |audio| {
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
                    if !audio.nodes.iter().any(|node| node.id == stream.node_id) {
                        return Err(ContractError::new(
                            "audio stream nodeId must reference a present audio node",
                        ));
                    }
                }
                Ok(())
            })
        }
        DesktopSystemUpdate::Media(capability) => {
            validate_capability(capability, "media", |media| {
                validate_unique_ids(&media.players, MAX_MEDIA_PLAYERS, "media player", |item| {
                    &item.id
                })?;
                for player in &media.players {
                    require_non_empty(&player.identity, "media player identity")?;
                    require_normalized(player.progress, "media player progress")?;
                }
                Ok(())
            })
        }
        DesktopSystemUpdate::Battery(capability) => {
            validate_capability(capability, "battery", |battery| {
                require_normalized(battery.level, "battery level")
            })
        }
        DesktopSystemUpdate::Brightness(capability) => {
            validate_capability(capability, "brightness", |brightness| {
                require_normalized(brightness.level, "brightness level")
            })
        }
        DesktopSystemUpdate::NightLight(capability) => {
            validate_capability(capability, "night light", |_| Ok(()))
        }
        DesktopSystemUpdate::Power(capability) => {
            validate_capability(capability, "power", |power| {
                let profiles: BTreeSet<_> = power.available_profiles.iter().collect();
                if power.available_profiles.is_empty()
                    || profiles.len() != power.available_profiles.len()
                    || !profiles.contains(&power.active_profile)
                {
                    return Err(ContractError::new(
                        "power profiles must be non-empty, unique, and contain activeProfile",
                    ));
                }
                Ok(())
            })
        }
        DesktopSystemUpdate::Osd(capability) => {
            validate_capability(capability, "OSD", validate_osd_snapshot)
        }
        DesktopSystemUpdate::Lock(capability) => {
            validate_capability(capability, "lock", |_| Ok(()))
        }
    }
}

fn validate_compositor_update(update: &DesktopCompositorUpdate) -> Result<(), ContractError> {
    match update {
        DesktopCompositorUpdate::Hyprland(capability) => {
            validate_capability(capability, "Hyprland", validate_hyprland_snapshot)
        }
        DesktopCompositorUpdate::Monitors(monitors) => {
            validate_unique_ids(monitors, MAX_MONITORS, "monitor", |item| &item.id)?;
            for monitor in monitors {
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
            if monitors.iter().filter(|item| item.focused).count() > 1 {
                return Err(ContractError::new(
                    "monitor update may contain at most one focused monitor",
                ));
            }
            Ok(())
        }
        DesktopCompositorUpdate::Workspaces(workspaces) => {
            validate_unique_ids(workspaces, MAX_WORKSPACES, "workspace", |item| &item.id)?;
            for workspace in workspaces {
                require_non_empty(&workspace.name, "workspace name")?;
                require_non_empty(&workspace.monitor_id, "workspace monitorId")?;
            }
            if workspaces.iter().filter(|item| item.focused).count() > 1 {
                return Err(ContractError::new(
                    "workspace update may contain at most one focused workspace",
                ));
            }
            Ok(())
        }
        DesktopCompositorUpdate::Windows(windows) => {
            validate_unique_ids(windows, MAX_WINDOWS, "window", |item| &item.id)?;
            for window in windows {
                require_non_empty(&window.application_id, "window applicationId")?;
                require_non_empty(&window.workspace_id, "window workspaceId")?;
            }
            if windows.iter().filter(|item| item.focused).count() > 1 {
                return Err(ContractError::new(
                    "window update may contain at most one focused window",
                ));
            }
            Ok(())
        }
    }
}

fn validate_hyprland_snapshot(hyprland: &HyprlandSnapshot) -> Result<(), ContractError> {
    validate_unique_ids(&hyprland.monitors, MAX_MONITORS, "monitor", |item| &item.id)?;
    validate_unique_ids(&hyprland.workspaces, MAX_WORKSPACES, "workspace", |item| {
        &item.id
    })?;
    validate_unique_ids(&hyprland.windows, MAX_WINDOWS, "window", |item| &item.id)?;
    if hyprland.monitors.iter().filter(|item| item.focused).count() > 1
        || hyprland
            .workspaces
            .iter()
            .filter(|item| item.focused)
            .count()
            > 1
        || hyprland.windows.iter().filter(|item| item.focused).count() > 1
    {
        return Err(ContractError::new(
            "Hyprland update has ambiguous focused records",
        ));
    }
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
        if !hyprland
            .monitors
            .iter()
            .any(|monitor| monitor.id == workspace.monitor_id)
        {
            return Err(ContractError::new(
                "workspace monitorId must reference a present monitor",
            ));
        }
    }
    for window in &hyprland.windows {
        require_non_empty(&window.application_id, "window applicationId")?;
        require_non_empty(&window.workspace_id, "window workspaceId")?;
        if !hyprland
            .workspaces
            .iter()
            .any(|workspace| workspace.id == window.workspace_id)
        {
            return Err(ContractError::new(
                "window workspaceId must reference a present workspace",
            ));
        }
    }
    Ok(())
}

fn validate_osd_snapshot(snapshot: &DesktopOsdSnapshot) -> Result<(), ContractError> {
    if snapshot.history.len() > MAX_OSD_HISTORY {
        return Err(ContractError::new("OSD history exceeds maximum of 500"));
    }
    if let Some(current) = &snapshot.current {
        validate_osd_event(&serialize(current, "OSD current event")?)?;
    }
    for event in &snapshot.history {
        validate_osd_event(&serialize(event, "OSD history event")?)?;
    }
    Ok(())
}

fn validate_notification_v3(notification: &NotificationDocument) -> Result<(), ContractError> {
    validate_notification_document(&serialize(notification, "notification")?)?;
    require_timestamp(&notification.created_at, "notification createdAt")?;
    let mut action_ids = BTreeSet::new();
    for action in &notification.actions {
        if !action_ids.insert(action.id.as_str()) {
            return Err(ContractError::new("notification action IDs must be unique"));
        }
    }
    Ok(())
}

fn validate_calendar_v3(snapshot: &CalendarSnapshot) -> Result<(), ContractError> {
    validate_calendar_snapshot(&serialize(snapshot, "calendar")?)?;
    require_timestamp(&snapshot.window_start, "calendar windowStart")?;
    require_timestamp(&snapshot.window_end, "calendar windowEnd")?;
    validate_unique_ids(&snapshot.events, usize::MAX, "calendar event", |event| {
        &event.id
    })?;
    for event in &snapshot.events {
        require_timestamp(&event.starts_at, "calendar event startsAt")?;
        require_timestamp(&event.ends_at, "calendar event endsAt")?;
    }
    Ok(())
}

fn validate_weather_v3(snapshot: &WeatherSnapshot) -> Result<(), ContractError> {
    validate_weather_snapshot(&serialize(snapshot, "weather")?)?;
    for point in &snapshot.forecast {
        require_timestamp(&point.at, "weather forecast at")?;
    }
    Ok(())
}

fn validate_utilities_snapshot(snapshot: &DesktopUtilitySnapshot) -> Result<(), ContractError> {
    validate_capability(&snapshot.tray_items, "tray items", |items| {
        validate_tray_items(items)
    })?;
    validate_capability(
        &snapshot.clipboard_entries,
        "clipboard entries",
        |entries| validate_clipboard_entries(entries),
    )?;
    validate_capability(&snapshot.recording, "recording", validate_recording_state)?;
    validate_capability(&snapshot.idle_inhibited, "idle inhibit", |_| Ok(()))?;
    validate_capability(&snapshot.game_mode, "GameMode", |_| Ok(()))?;
    validate_producer_availability(&snapshot.screenshot, "screenshot")?;
    validate_producer_availability(&snapshot.color_picker, "color picker")
}

fn validate_utility_update(update: &DesktopUtilityUpdate) -> Result<(), ContractError> {
    match update {
        DesktopUtilityUpdate::TrayItems(capability) => {
            validate_capability(capability, "tray items", |items| validate_tray_items(items))
        }
        DesktopUtilityUpdate::ClipboardEntries(capability) => {
            validate_capability(capability, "clipboard entries", |entries| {
                validate_clipboard_entries(entries)
            })
        }
        DesktopUtilityUpdate::Recording(capability) => {
            validate_capability(capability, "recording", validate_recording_state)
        }
        DesktopUtilityUpdate::IdleInhibited(capability) => {
            validate_capability(capability, "idle inhibit", |_| Ok(()))
        }
        DesktopUtilityUpdate::GameMode(capability) => {
            validate_capability(capability, "GameMode", |_| Ok(()))
        }
        DesktopUtilityUpdate::Screenshot(availability) => {
            validate_producer_availability(availability, "screenshot")
        }
        DesktopUtilityUpdate::ColorPicker(availability) => {
            validate_producer_availability(availability, "color picker")
        }
    }
}

fn validate_tray_items(items: &[TrayItem]) -> Result<(), ContractError> {
    validate_unique_ids(items, MAX_TRAY_ITEMS, "tray item", |item| &item.id)?;
    let mut menu_nodes = 0usize;
    for item in items {
        require_non_empty(&item.title, "tray item title")?;
        let mut menu_ids = BTreeSet::new();
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
    Ok(())
}

fn validate_clipboard_entries(entries: &[ClipboardEntry]) -> Result<(), ContractError> {
    validate_unique_ids(entries, MAX_CLIPBOARD_ENTRIES, "clipboard entry", |item| {
        &item.id
    })?;
    for entry in entries {
        require_non_empty(&entry.mime_type, "clipboard entry mimeType")?;
    }
    Ok(())
}

fn validate_producer_availability(
    availability: &ProducerAvailability,
    name: &str,
) -> Result<(), ContractError> {
    match availability.status {
        CapabilityAvailability::Available if availability.diagnostic.is_none() => Ok(()),
        CapabilityAvailability::Available => Err(ContractError::new(format!(
            "available {name} producer cannot contain a diagnostic"
        ))),
        _ => {
            let diagnostic = availability.diagnostic.as_ref().ok_or_else(|| {
                ContractError::new(format!("unavailable {name} producer requires a diagnostic"))
            })?;
            require_non_empty(&diagnostic.message, &format!("{name} diagnostic message"))
        }
    }
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
        DesktopCommand::System(command) => validate_system_command(command),
        DesktopCommand::Compositor(command) => validate_hyprland_command(command),
        DesktopCommand::Notification(command) => match command {
            NotificationCommand::SetDnd { .. } => Ok(()),
            NotificationCommand::Archive { notification_id }
            | NotificationCommand::InvokeAction {
                notification_id, ..
            } => {
                require_positive(*notification_id, "notification command id")?;
                if let NotificationCommand::InvokeAction { action_id, .. } = command {
                    require_stable_id(action_id, "notification actionId")?;
                }
                Ok(())
            }
        },
        DesktopCommand::Launcher(LauncherCommand::Launch(request)) => {
            validate_desktop_launch_request(&serialize(request, "desktop launch request")?)?;
            if request.desktop_id == ".desktop" {
                return Err(ContractError::new(
                    "launcher desktopId must have a non-empty basename",
                ));
            }
            if request.desktop_id.chars().any(char::is_control) {
                return Err(ContractError::new(
                    "launcher desktopId must not contain control characters",
                ));
            }
            require_maximum_length(&request.desktop_id, 256, "launcher desktopId")?;
            if let Some(action_id) = &request.action_id {
                require_bounded_non_empty(action_id, 256, "launcher actionId")?;
            }
            for resource in &request.resources {
                require_maximum_length(resource, 4_096, "launcher resource")?;
            }
            Ok(())
        }
        DesktopCommand::Appearance(command) => match command {
            AppearanceCommand::ApplyTheme { theme_id } => {
                require_stable_id(theme_id, "appearance themeId")
            }
            AppearanceCommand::SetWallpaper { wallpaper_id } => {
                require_stable_id(wallpaper_id, "appearance wallpaperId")
            }
            AppearanceCommand::SetReducedMotion { .. } | AppearanceCommand::SetOpaque { .. } => {
                Ok(())
            }
        },
        DesktopCommand::Utility(command) => match command {
            UtilityCommand::InvokeTrayMenu { item_id, menu_id } => {
                require_stable_id(item_id, "tray itemId")?;
                require_stable_id(menu_id, "tray menuId")
            }
            UtilityCommand::PasteClipboard { entry_id } => {
                require_stable_id(entry_id, "clipboard entryId")
            }
            UtilityCommand::StartRecording { output_id, .. }
            | UtilityCommand::Screenshot { output_id } => {
                require_stable_id(output_id, "utility outputId")
            }
            UtilityCommand::DeleteRecording { recording_id } => require_recording_id(recording_id),
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

fn require_recording_id(value: &StableId) -> Result<(), ContractError> {
    let value = value.as_str();
    let valid = value.starts_with("recording_")
        && value.ends_with(".mp4")
        && value.len() <= 96
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'));
    if valid {
        Ok(())
    } else {
        Err(ContractError::new(
            "utility recordingId must be a bounded recording_*.mp4 basename",
        ))
    }
}

fn validate_system_command(command: &DesktopSystemCommand) -> Result<(), ContractError> {
    match command {
        DesktopSystemCommand::Legacy(SystemMutation::AudioOutputDevice(id)) => {
            require_bounded_non_empty(id, 256, "system audio output device id")
        }
        DesktopSystemCommand::Legacy(_) => Ok(()),
        DesktopSystemCommand::Domain(mutation) => match mutation {
            DesktopSystemMutation::Network(command) => match command {
                NetworkCommand::ConnectWifi { access_point_id } => {
                    require_stable_id(access_point_id, "network accessPointId")
                }
                NetworkCommand::Disconnect { connection_id } => {
                    require_stable_id(connection_id, "network connectionId")
                }
                NetworkCommand::SetWifiEnabled { .. } | NetworkCommand::ScanWifi => Ok(()),
            },
            DesktopSystemMutation::Bluetooth(command) => match command {
                BluetoothCommand::Pair { device_id }
                | BluetoothCommand::Connect { device_id }
                | BluetoothCommand::Disconnect { device_id } => {
                    require_stable_id(device_id, "Bluetooth deviceId")
                }
                BluetoothCommand::SetPowered { .. } | BluetoothCommand::Scan => Ok(()),
            },
            DesktopSystemMutation::Audio(command) => match command {
                AudioCommand::SetDefaultNode { node_id }
                | AudioCommand::SetNodeMuted { node_id, .. } => {
                    require_stable_id(node_id, "audio nodeId")
                }
                AudioCommand::SetNodeVolume { node_id, level } => {
                    require_stable_id(node_id, "audio nodeId")?;
                    require_normalized(*level, "audio node level")
                }
                AudioCommand::SetStreamMuted { stream_id, .. } => {
                    require_stable_id(stream_id, "audio streamId")
                }
                AudioCommand::SetStreamVolume { stream_id, level } => {
                    require_stable_id(stream_id, "audio streamId")?;
                    require_normalized(*level, "audio stream level")
                }
            },
            DesktopSystemMutation::Media(MediaCommand::Transport { player_id, .. }) => {
                require_stable_id(player_id, "media playerId")
            }
            DesktopSystemMutation::Display(command) => match command {
                DisplayCommand::SetBrightness { output_id, level } => {
                    require_stable_id(output_id, "display outputId")?;
                    require_normalized(*level, "display brightness level")
                }
                DisplayCommand::SetNightLightEnabled { .. } => Ok(()),
            },
            DesktopSystemMutation::Power(_) => Ok(()),
        },
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
            require_stable_id(window_id, "Hyprland windowId")
        }
        HyprlandCommand::MoveWindowToWorkspace {
            window_id,
            workspace_id,
        } => {
            require_stable_id(window_id, "Hyprland windowId")?;
            require_stable_id(workspace_id, "Hyprland workspaceId")
        }
        HyprlandCommand::FocusWorkspace { workspace_id } => {
            require_stable_id(workspace_id, "Hyprland workspaceId")
        }
        HyprlandCommand::MoveWorkspaceToMonitor {
            workspace_id,
            monitor_id,
        } => {
            require_stable_id(workspace_id, "Hyprland workspaceId")?;
            require_stable_id(monitor_id, "Hyprland monitorId")
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
        DesktopResultStatus::Failed => {
            let diagnostic = result
                .diagnostic
                .as_ref()
                .ok_or_else(|| ContractError::new("failed desktop result requires a diagnostic"))?;
            require_non_empty(&diagnostic.message, "desktop result diagnostic message")
        }
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

fn require_bounded_non_empty(value: &str, maximum: usize, name: &str) -> Result<(), ContractError> {
    require_non_empty(value, name)?;
    require_maximum_length(value, maximum, name)
}

fn require_maximum_length(value: &str, maximum: usize, name: &str) -> Result<(), ContractError> {
    if value.chars().count() > maximum {
        Err(ContractError::new(format!(
            "{name} exceeds maximum length of {maximum}"
        )))
    } else {
        Ok(())
    }
}

fn require_stable_id(value: &StableId, name: &str) -> Result<(), ContractError> {
    require_bounded_non_empty(value.as_str(), 256, name)
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
    if !is_canonical_utc_rfc3339(value) {
        Err(ContractError::new(format!(
            "{name} must be canonical UTC RFC3339"
        )))
    } else {
        Ok(())
    }
}

fn is_canonical_utc_rfc3339(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || *bytes.last().unwrap_or(&0) != b'Z'
    {
        return false;
    }
    let fraction = &bytes[19..bytes.len() - 1];
    if !fraction.is_empty()
        && (fraction[0] != b'.'
            || fraction.len() == 1
            || !fraction[1..].iter().all(u8::is_ascii_digit))
    {
        return false;
    }
    for range in [0..4, 5..7, 8..10, 11..13, 14..16, 17..19] {
        if !bytes[range].iter().all(u8::is_ascii_digit) {
            return false;
        }
    }

    let parse = |range: std::ops::Range<usize>| {
        std::str::from_utf8(&bytes[range])
            .ok()
            .and_then(|part| part.parse::<u32>().ok())
    };
    let (Some(year), Some(month), Some(day), Some(hour), Some(minute), Some(second)) = (
        parse(0..4),
        parse(5..7),
        parse(8..10),
        parse(11..13),
        parse(14..16),
        parse(17..19),
    ) else {
        return false;
    };
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        2 if leap_year => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=days).contains(&day)
}

fn parse<T: for<'de> Deserialize<'de>>(input: &str, name: &str) -> Result<T, ContractError> {
    serde_json::from_str(input)
        .map_err(|error| ContractError::new(format!("invalid {name}: {error}")))
}

fn serialize<T: Serialize>(value: &T, name: &str) -> Result<String, ContractError> {
    serde_json::to_string(value)
        .map_err(|error| ContractError::new(format!("invalid {name}: {error}")))
}
