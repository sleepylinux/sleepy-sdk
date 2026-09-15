//! Opt-in capture v1 endpoint. This does not extend the closed desktop v3 stream.
//!
//! PNG paths are session-local temporary results, retained until history eviction
//! or session end. Consumers must export a result to keep it.
//! These validators check syntax, not filesystem ownership, PNG bytes, or consent:
//! the session provider must enforce those before publishing completion.
use serde::{Deserialize, Deserializer, Serialize};

use crate::ContractError;

pub const CAPTURE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureRequest {
    pub schema_version: u32,
    pub command: CaptureCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum CaptureCommand {
    Begin { job_id: String, output_id: String },
    Status { job_id: String },
    Cancel { job_id: String },
    Capabilities,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureReply {
    pub schema_version: u32,
    pub payload: CapturePayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum CapturePayload {
    Job {
        job: CaptureJob,
    },
    Capabilities {
        screenshot: bool,
        color_picker: bool,
    },
    Error {
        diagnostic: CaptureDiagnostic,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureJob {
    pub job_id: String,
    pub output_id: String,
    pub state: CaptureState,
    #[serde(
        default,
        deserialize_with = "non_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub result: Option<CapturePng>,
    #[serde(
        default,
        deserialize_with = "non_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub diagnostic: Option<CaptureDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptureState {
    AwaitingConsent,
    Capturing,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CapturePng {
    pub path: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureDiagnostic {
    pub code: CaptureErrorCode,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CaptureErrorCode {
    Unavailable,
    Busy,
    NotFound,
    InvalidRequest,
    OutputUnavailable,
    ConsentTimedOut,
    CaptureFailed,
    Cancelled,
}

fn non_null<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

fn require(condition: bool, message: &str) -> Result<(), ContractError> {
    if condition {
        Ok(())
    } else {
        Err(ContractError::new(message))
    }
}

fn job_id(id: &str) -> Result<(), ContractError> {
    require(
        uuid::Uuid::parse_str(id).is_ok_and(|uuid| uuid.hyphenated().to_string() == id),
        "capture jobId must be a canonical UUID",
    )
}

fn output_id(id: &str) -> Result<(), ContractError> {
    require(
        id.strip_prefix("output:").is_some_and(|name| {
            !name.is_empty()
                && name.len() <= 128
                && name
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
        }),
        "invalid capture outputId",
    )
}

fn diagnostic(value: &CaptureDiagnostic) -> Result<(), ContractError> {
    require(
        !value.message.is_empty()
            && value.message.len() <= 256
            && !value.message.chars().any(char::is_control),
        "capture diagnostic must be bounded plain text",
    )
}

pub fn validate_capture_request(input: &str) -> Result<CaptureRequest, ContractError> {
    require(input.len() <= 4096, "capture request is too large")?;
    let request: CaptureRequest =
        serde_json::from_str(input).map_err(|_| ContractError::new("invalid capture request"))?;
    require(
        request.schema_version == CAPTURE_SCHEMA_VERSION,
        "unsupported capture schema",
    )?;
    match &request.command {
        CaptureCommand::Begin {
            job_id: id,
            output_id: output,
        } => {
            job_id(id)?;
            output_id(output)?;
        }
        CaptureCommand::Status { job_id: id } | CaptureCommand::Cancel { job_id: id } => {
            job_id(id)?
        }
        CaptureCommand::Capabilities => {}
    }
    Ok(request)
}

pub fn validate_capture_reply(input: &str) -> Result<CaptureReply, ContractError> {
    require(input.len() <= 4096, "capture reply is too large")?;
    let reply: CaptureReply =
        serde_json::from_str(input).map_err(|_| ContractError::new("invalid capture reply"))?;
    require(
        reply.schema_version == CAPTURE_SCHEMA_VERSION,
        "unsupported capture schema",
    )?;
    match &reply.payload {
        CapturePayload::Capabilities { color_picker, .. } => {
            require(!color_picker, "capture v1 does not implement color picking")?
        }
        CapturePayload::Error { diagnostic: value } => diagnostic(value)?,
        CapturePayload::Job { job } => {
            job_id(&job.job_id)?;
            output_id(&job.output_id)?;
            require(
                job.result.is_some() == (job.state == CaptureState::Completed),
                "only completed capture has a PNG result",
            )?;
            require(
                job.diagnostic.is_some() == (job.state == CaptureState::Failed),
                "only failed capture has a diagnostic",
            )?;
            if let Some(value) = &job.diagnostic {
                diagnostic(value)?;
            }
            if let Some(result) = &job.result {
                let parts: Vec<_> = result.path.split('/').collect();
                let uid_valid = parts.get(3).is_some_and(|uid| {
                    uid.parse::<u32>()
                        .is_ok_and(|number| number.to_string() == *uid)
                });
                require(
                    parts.len() == 7
                        && parts[..3] == ["", "run", "user"]
                        && uid_valid
                        && parts[4..6] == ["sleepy", "captures"]
                        && parts[6] == format!("screenshot-{}.png", job.job_id),
                    "capture result must name this job in the runtime capture directory",
                )?;
                require(
                    result.mime_type == "image/png"
                        && (1..=32768).contains(&result.width)
                        && (1..=32768).contains(&result.height),
                    "capture result must be a bounded PNG",
                )?;
            }
        }
    }
    Ok(reply)
}
