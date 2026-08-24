use serde::{Deserialize, Serialize};

use crate::{CacheStatus, CapabilityFailure, ContractError, ProviderStatus, WIRE_SCHEMA_VERSION};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DesktopLaunchRequest {
    pub schema_version: u32,
    pub desktop_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_id: Option<String>,
    pub resources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OsdEvent {
    pub schema_version: u32,
    pub output_id: String,
    pub kind: OsdKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub muted: Option<bool>,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OsdKind {
    Volume,
    Microphone,
    Brightness,
    Media,
    PowerProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalendarSnapshot {
    pub schema_version: u32,
    pub provider_id: String,
    pub window_start: String,
    pub window_end: String,
    pub events: Vec<CalendarEvent>,
    pub source_errors: Vec<CalendarSourceError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalendarEvent {
    pub id: String,
    pub summary: String,
    pub starts_at: String,
    pub ends_at: String,
    pub all_day: bool,
    pub source_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CalendarSourceError {
    pub source_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WeatherSnapshot {
    pub schema_version: u32,
    pub provider_id: String,
    pub location: WeatherLocation,
    pub status: ProviderStatus,
    pub cache: CacheStatus,
    pub attribution: String,
    pub forecast: Vec<ForecastPoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<CapabilityFailure>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WeatherLocation {
    pub display_name: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ForecastPoint {
    pub at: String,
    pub temperature_c: f64,
    pub symbol: String,
}

pub trait CalendarProvider {
    fn snapshot(
        &self,
        window_start: &str,
        window_end: &str,
    ) -> Result<CalendarSnapshot, ContractError>;
}

pub trait WeatherProvider {
    fn snapshot(&self, location: &WeatherLocation) -> Result<WeatherSnapshot, ContractError>;
}

pub fn validate_desktop_launch_request(input: &str) -> Result<DesktopLaunchRequest, ContractError> {
    let request: DesktopLaunchRequest = parse(input, "desktop launch request")?;
    require_wire_version(request.schema_version, "desktop launch request")?;
    if !request.desktop_id.ends_with(".desktop")
        || request.desktop_id.contains('/')
        || request.desktop_id.contains('\\')
        || request.desktop_id.contains("..")
    {
        return Err(ContractError::new(
            "desktopId must be an indexed Desktop Entry identifier",
        ));
    }
    if request
        .action_id
        .as_deref()
        .is_some_and(|value| value.is_empty() || value.trim() != value)
    {
        return Err(ContractError::new(
            "desktop actionId must be non-empty and unpadded",
        ));
    }
    for resource in &request.resources {
        if resource.is_empty() || resource.contains('\0') {
            return Err(ContractError::new("desktop resource must be non-empty"));
        }
    }
    Ok(request)
}

pub fn validate_osd_event(input: &str) -> Result<OsdEvent, ContractError> {
    let event: OsdEvent = parse(input, "OSD event")?;
    require_wire_version(event.schema_version, "OSD event")?;
    require_non_empty(&event.output_id, "OSD outputId")?;
    require_non_empty(&event.label, "OSD label")?;
    if event
        .level
        .is_some_and(|level| !(0.0..=1.0).contains(&level))
    {
        return Err(ContractError::new("OSD level must be normalized"));
    }
    match event.kind {
        OsdKind::Volume | OsdKind::Microphone | OsdKind::Brightness if event.level.is_none() => {
            Err(ContractError::new("level OSD event requires level"))
        }
        OsdKind::Brightness if event.muted.is_some() => {
            Err(ContractError::new("brightness OSD cannot contain muted"))
        }
        _ => Ok(event),
    }
}

pub fn validate_calendar_snapshot(input: &str) -> Result<CalendarSnapshot, ContractError> {
    let snapshot: CalendarSnapshot = parse(input, "calendar snapshot")?;
    require_wire_version(snapshot.schema_version, "calendar snapshot")?;
    require_non_empty(&snapshot.provider_id, "calendar providerId")?;
    require_timestamp(&snapshot.window_start, "calendar windowStart")?;
    require_timestamp(&snapshot.window_end, "calendar windowEnd")?;
    if snapshot.window_start >= snapshot.window_end {
        return Err(ContractError::new("calendar window must be ordered"));
    }
    for event in &snapshot.events {
        require_non_empty(&event.id, "calendar event id")?;
        require_non_empty(&event.summary, "calendar event summary")?;
        require_non_empty(&event.source_id, "calendar event sourceId")?;
        require_timestamp(&event.starts_at, "calendar event startsAt")?;
        require_timestamp(&event.ends_at, "calendar event endsAt")?;
        if event.starts_at >= event.ends_at {
            return Err(ContractError::new(
                "calendar event interval must be ordered",
            ));
        }
    }
    for error in &snapshot.source_errors {
        require_non_empty(&error.source_id, "calendar source error id")?;
        require_non_empty(&error.message, "calendar source error message")?;
    }
    Ok(snapshot)
}

pub fn validate_weather_snapshot(input: &str) -> Result<WeatherSnapshot, ContractError> {
    let snapshot: WeatherSnapshot = parse(input, "weather snapshot")?;
    require_wire_version(snapshot.schema_version, "weather snapshot")?;
    require_non_empty(&snapshot.provider_id, "weather providerId")?;
    require_non_empty(
        &snapshot.location.display_name,
        "weather location displayName",
    )?;
    require_non_empty(&snapshot.attribution, "weather attribution")?;
    if !(-90.0..=90.0).contains(&snapshot.location.latitude)
        || !(-180.0..=180.0).contains(&snapshot.location.longitude)
    {
        return Err(ContractError::new("weather coordinates are out of range"));
    }
    match snapshot.status {
        ProviderStatus::Online if snapshot.diagnostic.is_some() => {
            return Err(ContractError::new(
                "online weather snapshot cannot contain a diagnostic",
            ));
        }
        ProviderStatus::Offline | ProviderStatus::Error if snapshot.diagnostic.is_none() => {
            return Err(ContractError::new(
                "unavailable weather snapshot requires a diagnostic",
            ));
        }
        _ => {}
    }
    for point in &snapshot.forecast {
        require_timestamp(&point.at, "forecast timestamp")?;
        require_non_empty(&point.symbol, "forecast symbol")?;
        if !point.temperature_c.is_finite() {
            return Err(ContractError::new("forecast temperature must be finite"));
        }
    }
    Ok(snapshot)
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

fn require_non_empty(value: &str, name: &str) -> Result<(), ContractError> {
    if value.is_empty() || value.trim() != value {
        Err(ContractError::new(format!(
            "{name} must be non-empty and unpadded"
        )))
    } else {
        Ok(())
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
