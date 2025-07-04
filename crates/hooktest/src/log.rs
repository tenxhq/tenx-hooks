use crate::color::ColorMode;
use anyhow::{Result, bail};
use code_hooks::{
    HookResponse, Input, Notification, PostToolUse, PostToolUseOutput, PreToolUse,
    PreToolUseOutput, Stop, SubagentStop, TranscriptReader,
};
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize)]
struct LogEntry<'a, T> {
    event: String,
    timestamp: u64,
    data: &'a T,
}

pub fn run_log_hook(
    event: String,
    filepath: String,
    transcript_path: Option<String>,
    _color_mode: ColorMode,
) -> Result<()> {
    // Parse the input based on event type and handle it
    match event.as_str() {
        "pretool" => {
            let input = PreToolUse::read()?;
            log_event("pretool", &input, &filepath)?;
            if let Some(transcript_path) = transcript_path {
                process_transcript(&input, &transcript_path)?;
            }
            PreToolUseOutput::passthrough().respond()
        }
        "posttool" => {
            let input = PostToolUse::read()?;
            log_event("posttool", &input, &filepath)?;
            if let Some(transcript_path) = transcript_path {
                process_transcript(&input, &transcript_path)?;
            }
            PostToolUseOutput::passthrough().respond()
        }
        "notification" => {
            let input = Notification::read()?;
            log_event("notification", &input, &filepath)?;
            if let Some(transcript_path) = transcript_path {
                process_transcript(&input, &transcript_path)?;
            }
            Notification::passthrough().respond()
        }
        "stop" => {
            let input = Stop::read()?;
            log_event("stop", &input, &filepath)?;
            if let Some(transcript_path) = transcript_path {
                process_transcript(&input, &transcript_path)?;
            }
            input.allow().respond()
        }
        "subagentstop" => {
            let input = SubagentStop::read()?;
            log_event("subagentstop", &input, &filepath)?;
            if let Some(transcript_path) = transcript_path {
                process_transcript(&input, &transcript_path)?;
            }
            input.allow().respond()
        }
        _ => bail!(
            "Unknown event type: {}. Must be one of: pretool, posttool, notification, stop, subagentstop",
            event
        ),
    }
}

fn log_event<T: serde::Serialize>(event_name: &str, data: &T, filepath: &str) -> Result<()> {
    let log_entry = LogEntry {
        event: event_name.to_string(),
        timestamp: get_timestamp(),
        data,
    };
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(filepath)?;
    writeln!(file, "{}", serde_json::to_string(&log_entry)?)?;
    Ok(())
}

fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn process_transcript<T>(input: &T, output_path: &str) -> Result<()>
where
    T: TranscriptReader,
{
    let transcript_entries = input.read_transcript()?;
    let mut file = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(output_path)?;

    for entry in transcript_entries {
        writeln!(file, "{}", serde_json::to_string(&entry)?)?;
    }

    Ok(())
}
