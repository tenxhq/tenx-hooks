use crate::error::Result;
use serde::Deserialize;
use std::io::{self, Read as IoRead};

/// Input structure for Notification hooks.
///
/// Notification hooks run when Claude Code sends notifications, allowing
/// you to customize how you receive alerts (e.g., when Claude needs input
/// or permission to run something).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Notification {
    /// Unique identifier for the current Claude Code session
    pub session_id: String,
    /// Path to the conversation transcript JSON file
    pub transcript_path: String,
    /// The notification message content
    pub message: String,
    /// The notification title (typically "Claude Code")
    pub title: String,
}

/// Input structure for Stop hooks.
///
/// Stop hooks run when Claude Code has finished responding. They can
/// block Claude from stopping and request continuation.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Stop {
    /// Unique identifier for the current Claude Code session
    pub session_id: String,
    /// Path to the conversation transcript JSON file
    pub transcript_path: String,
    /// True when Claude Code is already continuing as a result of a stop hook.
    /// Check this to prevent infinite loops.
    pub stop_hook_active: bool,
}

/// Trait for hook input types that can be read from stdin.
///
/// This trait provides a standard way to read hook inputs by:
/// 1. Reading all content from stdin
/// 2. Parsing it as JSON
/// 3. Deserializing to the appropriate type
///
/// # Example
///
/// ```rust,no_run
/// use tenx_hooks::{Input, PreToolUse};
///
/// let input = PreToolUse::read().expect("Failed to read input");
/// println!("Tool name: {}", input.tool_name);
/// ```
pub trait Input: for<'de> Deserialize<'de> + Sized {
    /// Read and parse input from stdin.
    fn read() -> Result<Self> {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        let parsed = serde_json::from_str(&buffer)?;
        Ok(parsed)
    }
}

impl Input for Notification {}
impl Input for Stop {}
