use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::ContractError;

const MODIFIERS: [&str; 4] = ["Mod", "Ctrl", "Alt", "Shift"];
const PACKAGED_RESERVED_KEYBINDINGS: [(&str, &str); 1] = [("recovery.shell", "Mod+Shift+Escape")];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticAction {
    TerminalOpen,
    LauncherOpen,
    WindowClose,
    WindowFocusLeft,
    WindowFocusRight,
    WindowFocusUp,
    WindowFocusDown,
    WorkspacePrevious,
    WorkspaceNext,
    ControlCenterToggle,
    SessionLock,
    SessionLogout,
    SessionReboot,
    SessionPowerOff,
    SessionPower,
    MediaPlayPause,
    MediaNext,
    MediaPrevious,
    VolumeUp,
    VolumeDown,
    VolumeToggleMute,
    MicrophoneToggleMute,
    BrightnessUp,
    BrightnessDown,
}

pub const KNOWN_SEMANTIC_ACTIONS: &[SemanticAction] = &[
    SemanticAction::TerminalOpen,
    SemanticAction::LauncherOpen,
    SemanticAction::WindowClose,
    SemanticAction::WindowFocusLeft,
    SemanticAction::WindowFocusRight,
    SemanticAction::WindowFocusUp,
    SemanticAction::WindowFocusDown,
    SemanticAction::WorkspacePrevious,
    SemanticAction::WorkspaceNext,
    SemanticAction::ControlCenterToggle,
    SemanticAction::SessionLock,
    SemanticAction::SessionLogout,
    SemanticAction::SessionReboot,
    SemanticAction::SessionPowerOff,
    SemanticAction::SessionPower,
    SemanticAction::MediaPlayPause,
    SemanticAction::MediaNext,
    SemanticAction::MediaPrevious,
    SemanticAction::VolumeUp,
    SemanticAction::VolumeDown,
    SemanticAction::VolumeToggleMute,
    SemanticAction::MicrophoneToggleMute,
    SemanticAction::BrightnessUp,
    SemanticAction::BrightnessDown,
];

impl SemanticAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TerminalOpen => "app.terminal.open",
            Self::LauncherOpen => "launcher.open",
            Self::WindowClose => "window.close",
            Self::WindowFocusLeft => "window.focus.left",
            Self::WindowFocusRight => "window.focus.right",
            Self::WindowFocusUp => "window.focus.up",
            Self::WindowFocusDown => "window.focus.down",
            Self::WorkspacePrevious => "workspace.previous",
            Self::WorkspaceNext => "workspace.next",
            Self::ControlCenterToggle => "surface.controlCenter.toggle",
            Self::SessionLock => "session.lock",
            Self::SessionLogout => "session.logout",
            Self::SessionReboot => "session.reboot",
            Self::SessionPowerOff => "session.powerOff",
            Self::SessionPower => "session.power",
            Self::MediaPlayPause => "media.playPause",
            Self::MediaNext => "media.next",
            Self::MediaPrevious => "media.previous",
            Self::VolumeUp => "audio.volume.up",
            Self::VolumeDown => "audio.volume.down",
            Self::VolumeToggleMute => "audio.volume.toggleMute",
            Self::MicrophoneToggleMute => "audio.microphone.toggleMute",
            Self::BrightnessUp => "display.brightness.up",
            Self::BrightnessDown => "display.brightness.down",
        }
    }
}

impl TryFrom<&str> for SemanticAction {
    type Error = ContractError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        KNOWN_SEMANTIC_ACTIONS
            .iter()
            .copied()
            .find(|action| action.as_str() == value)
            .ok_or_else(|| ContractError::new(format!("unknown semantic action {value}")))
    }
}

impl fmt::Display for SemanticAction {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for SemanticAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SemanticAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::try_from(value.as_str()).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConflictKind {
    Duplicate,
    Reserved,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeybindingConflict {
    pub kind: ConflictKind,
    pub accelerator: String,
    pub actions: Vec<String>,
}

impl fmt::Display for KeybindingConflict {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:?} keybinding conflict for {}: {}",
            self.kind,
            self.accelerator,
            self.actions.join(", ")
        )
    }
}

pub fn canonicalize_accelerator(input: &str) -> Result<String, ContractError> {
    let input = input.trim();
    if input.chars().any(char::is_whitespace) {
        return Err(ContractError::new(
            "accelerator must not contain internal whitespace",
        ));
    }

    let mut modifiers = BTreeSet::new();
    let mut key = None;

    for component in input.split('+') {
        if component.is_empty() {
            return Err(ContractError::new(
                "accelerator components must not be blank",
            ));
        }

        if let Some(modifier) = MODIFIERS
            .iter()
            .find(|modifier| modifier.eq_ignore_ascii_case(component))
        {
            if !modifiers.insert(*modifier) {
                return Err(ContractError::new(format!(
                    "accelerator contains duplicate modifier {modifier}"
                )));
            }
        } else {
            if !component
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                return Err(ContractError::new(
                    "accelerator key must contain only ASCII letters, digits, or underscore",
                ));
            }
            if key.replace(component).is_some() {
                return Err(ContractError::new(
                    "accelerator must contain exactly one key",
                ));
            }
        }
    }

    let key = key.ok_or_else(|| ContractError::new("accelerator must contain exactly one key"))?;
    let key = canonicalize_key(key);

    let mut components: Vec<&str> = MODIFIERS
        .iter()
        .copied()
        .filter(|modifier| modifiers.contains(modifier))
        .collect();
    components.push(&key);

    Ok(components.join("+"))
}

pub fn validate_keybindings(bindings: &BTreeMap<String, String>) -> Result<(), ContractError> {
    let reserved_bindings = packaged_reserved_keybindings();
    validate_keybindings_with_reserved(bindings, &reserved_bindings)
        .map_err(|conflict| ContractError::new(conflict.to_string()))
}

pub fn packaged_reserved_keybindings() -> BTreeMap<String, String> {
    PACKAGED_RESERVED_KEYBINDINGS
        .iter()
        .map(|(action, accelerator)| ((*action).to_string(), (*accelerator).to_string()))
        .collect()
}

pub fn validate_keybindings_with_reserved(
    bindings: &BTreeMap<String, String>,
    reserved_bindings: &BTreeMap<String, String>,
) -> Result<(), KeybindingConflict> {
    let mut chords: BTreeMap<String, (String, bool)> = BTreeMap::new();

    for (action, binding) in reserved_bindings {
        let canonical = canonicalize_accelerator(binding).map_err(|_| KeybindingConflict {
            kind: ConflictKind::Invalid,
            accelerator: binding.clone(),
            actions: vec![action.clone()],
        })?;
        if let Some((existing_action, _)) = chords.insert(canonical.clone(), (action.clone(), true))
        {
            return Err(KeybindingConflict {
                kind: ConflictKind::Reserved,
                accelerator: canonical,
                actions: vec![existing_action, action.clone()],
            });
        }
    }

    for (action, binding) in bindings {
        let canonical = canonicalize_accelerator(binding).map_err(|_| KeybindingConflict {
            kind: ConflictKind::Invalid,
            accelerator: binding.clone(),
            actions: vec![action.clone()],
        })?;
        if SemanticAction::try_from(action.as_str()).is_err() {
            return Err(KeybindingConflict {
                kind: ConflictKind::Invalid,
                accelerator: canonical,
                actions: vec![action.clone()],
            });
        }

        if let Some((existing_action, reserved)) =
            chords.insert(canonical.clone(), (action.clone(), false))
        {
            return Err(KeybindingConflict {
                kind: if reserved {
                    ConflictKind::Reserved
                } else {
                    ConflictKind::Duplicate
                },
                accelerator: canonical,
                actions: vec![existing_action, action.clone()],
            });
        }
    }

    Ok(())
}

fn canonicalize_key(key: &str) -> String {
    const NAMED_KEYS: &[(&str, &str)] = &[
        ("escape", "Escape"),
        ("space", "Space"),
        ("return", "Return"),
        ("enter", "Return"),
        ("tab", "Tab"),
        ("backspace", "BackSpace"),
        ("back_space", "BackSpace"),
        ("delete", "Delete"),
        ("insert", "Insert"),
        ("home", "Home"),
        ("end", "End"),
        ("pageup", "Page_Up"),
        ("page_up", "Page_Up"),
        ("prior", "Page_Up"),
        ("pagedown", "Page_Down"),
        ("page_down", "Page_Down"),
        ("next", "Page_Down"),
        ("left", "Left"),
        ("right", "Right"),
        ("up", "Up"),
        ("down", "Down"),
        ("xf86audioplay", "XF86AudioPlay"),
        ("xf86audiopause", "XF86AudioPause"),
        ("xf86audiostop", "XF86AudioStop"),
        ("xf86audionext", "XF86AudioNext"),
        ("xf86audioprev", "XF86AudioPrev"),
        ("xf86audioraisevolume", "XF86AudioRaiseVolume"),
        ("xf86audiolowervolume", "XF86AudioLowerVolume"),
        ("xf86audiomute", "XF86AudioMute"),
        ("xf86audiomicmute", "XF86AudioMicMute"),
        ("xf86monbrightnessup", "XF86MonBrightnessUp"),
        ("xf86monbrightnessdown", "XF86MonBrightnessDown"),
    ];

    if key.chars().count() == 1 {
        key.to_uppercase()
    } else if let Some(number) = key
        .strip_prefix(['f', 'F'])
        .and_then(|digits| digits.parse::<u8>().ok())
        .filter(|number| (1..=24).contains(number))
    {
        format!("F{number}")
    } else if let Some((_, canonical)) = NAMED_KEYS
        .iter()
        .find(|(alias, _)| alias.eq_ignore_ascii_case(key))
    {
        (*canonical).to_string()
    } else {
        key.to_string()
    }
}
