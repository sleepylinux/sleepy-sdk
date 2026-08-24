use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

use crate::ContractError;

const MODIFIERS: [&str; 4] = ["Mod", "Ctrl", "Alt", "Shift"];

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
pub enum KeybindingConflictKind {
    Duplicate,
    Reserved,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KeybindingConflict {
    pub kind: KeybindingConflictKind,
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
    let mut modifiers = BTreeSet::new();
    let mut key = None;

    for component in input.split('+') {
        let component = component.trim();
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
        } else if key.replace(component).is_some() {
            return Err(ContractError::new(
                "accelerator must contain exactly one key",
            ));
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
    validate_keybindings_with_reserved(bindings, &BTreeMap::new())
        .map_err(|conflict| ContractError::new(conflict.to_string()))
}

pub fn validate_keybindings_with_reserved(
    bindings: &BTreeMap<String, String>,
    reserved_bindings: &BTreeMap<String, String>,
) -> Result<(), KeybindingConflict> {
    let mut chords: BTreeMap<String, (String, bool)> = BTreeMap::new();

    for (action, binding) in reserved_bindings {
        let canonical = canonicalize_accelerator(binding).map_err(|_| KeybindingConflict {
            kind: KeybindingConflictKind::Invalid,
            accelerator: binding.clone(),
            actions: vec![action.clone()],
        })?;
        if let Some((existing_action, _)) = chords.insert(canonical.clone(), (action.clone(), true))
        {
            return Err(KeybindingConflict {
                kind: KeybindingConflictKind::Reserved,
                accelerator: canonical,
                actions: vec![existing_action, action.clone()],
            });
        }
    }

    for (action, binding) in bindings {
        let canonical = canonicalize_accelerator(binding).map_err(|_| KeybindingConflict {
            kind: KeybindingConflictKind::Invalid,
            accelerator: binding.clone(),
            actions: vec![action.clone()],
        })?;
        if SemanticAction::try_from(action.as_str()).is_err() {
            return Err(KeybindingConflict {
                kind: KeybindingConflictKind::Invalid,
                accelerator: canonical,
                actions: vec![action.clone()],
            });
        }

        if let Some((existing_action, reserved)) =
            chords.insert(canonical.clone(), (action.clone(), false))
        {
            return Err(KeybindingConflict {
                kind: if reserved {
                    KeybindingConflictKind::Reserved
                } else {
                    KeybindingConflictKind::Duplicate
                },
                accelerator: canonical,
                actions: vec![existing_action, action.clone()],
            });
        }
    }

    Ok(())
}

fn canonicalize_key(key: &str) -> String {
    const NAMED_KEYS: [&str; 15] = [
        "Escape",
        "Space",
        "Return",
        "Enter",
        "Tab",
        "Backspace",
        "Delete",
        "Home",
        "End",
        "PageUp",
        "PageDown",
        "Left",
        "Right",
        "Up",
        "Down",
    ];

    if key.chars().count() == 1 {
        key.to_uppercase()
    } else if let Some(named) = NAMED_KEYS
        .iter()
        .find(|named| named.eq_ignore_ascii_case(key))
    {
        (*named).to_string()
    } else {
        key.to_string()
    }
}
