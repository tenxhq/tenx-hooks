//! A Rust library for building hooks for Claude Code.
//!
//! Claude Code hooks are user-defined shell commands that execute at various points
//! in Claude Code's lifecycle. They provide deterministic control over Claude Code's
//! behavior, ensuring certain actions always happen rather than relying on the LLM
//! to choose to run them.
//!
//! # Example
//!
//! ```rust,no_run
//! use tenx_hooks::{Hook, PreToolUseOutput, Decision};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let hook = Hook::new();
//!     let input = hook.pre_tool_use()?;
//!     
//!     if input.tool_name == "Bash" {
//!         if let Some(command) = input.tool_input.get("command").and_then(|v| v.as_str()) {
//!             if command.contains("rm -rf") {
//!                 hook.respond(PreToolUseOutput::block("Dangerous command detected"));
//!                 return Ok(());
//!             }
//!         }
//!     }
//!     
//!     hook.respond(PreToolUseOutput::approve("Command validated"));
//!     Ok(())
//! }
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, Read};

/// Main hook interface for interacting with Claude Code.
///
/// The `Hook` struct provides methods to read input from stdin and send
/// responses to stdout, handling all JSON serialization/deserialization
/// automatically.
pub struct Hook;

impl Hook {
    /// Create a new Hook instance
    pub fn new() -> Self {
        Hook
    }

    /// Read and parse PreToolUse input from stdin
    pub fn pre_tool_use(&self) -> Result<PreToolUseInput, HookError> {
        self.read_input()
    }

    /// Read and parse PostToolUse input from stdin
    pub fn post_tool_use(&self) -> Result<PostToolUseInput, HookError> {
        self.read_input()
    }

    /// Read and parse Notification input from stdin
    pub fn notification(&self) -> Result<NotificationInput, HookError> {
        self.read_input()
    }

    /// Read and parse Stop input from stdin
    pub fn stop(&self) -> Result<StopInput, HookError> {
        self.read_input()
    }

    /// Send a response to stdout
    pub fn respond<T: Serialize>(&self, output: T) {
        match serde_json::to_string(&output) {
            Ok(json) => {
                println!("{json}");
            }
            Err(e) => {
                eprintln!("Failed to serialize output: {e}");
                std::process::exit(1);
            }
        }
    }

    /// Internal method to read and parse JSON from stdin
    fn read_input<T: for<'de> Deserialize<'de>>(&self) -> Result<T, HookError> {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(HookError::IoError)?;

        serde_json::from_str(&buffer).map_err(HookError::ParseError)
    }
}

impl Default for Hook {
    fn default() -> Self {
        Self::new()
    }
}

/// Error types for hook operations
#[derive(Debug)]
pub enum HookError {
    /// Error reading from stdin
    IoError(io::Error),
    /// Error parsing JSON input
    ParseError(serde_json::Error),
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookError::IoError(e) => write!(f, "IO error: {e}"),
            HookError::ParseError(e) => write!(f, "Parse error: {e}"),
        }
    }
}

impl std::error::Error for HookError {}

/// Decision type for approve/block operations.
///
/// Used in PreToolUse, PostToolUse, and Stop hooks to control execution flow.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    /// Approve the operation (PreToolUse only - bypasses permission system)
    Approve,
    /// Block the operation and provide feedback to Claude
    Block,
}

/// Common fields shared by all input types.
///
/// Note: This struct is not used directly in the current implementation,
/// but documents the common fields present in all hook inputs.
#[derive(Debug, Deserialize)]
pub struct CommonInput {
    /// Unique identifier for the current Claude Code session
    pub session_id: String,
    /// Path to the conversation transcript JSON file
    pub transcript_path: String,
}

/// Input structure for PreToolUse hooks.
///
/// PreToolUse hooks run after Claude creates tool parameters but before
/// processing the tool call. They can approve or block the operation.
#[derive(Debug, Deserialize)]
pub struct PreToolUseInput {
    /// Unique identifier for the current Claude Code session
    pub session_id: String,
    /// Path to the conversation transcript JSON file
    pub transcript_path: String,
    /// Name of the tool being called (e.g., "Bash", "Write", "Edit")
    pub tool_name: String,
    /// Tool-specific input parameters. The exact schema depends on the tool.
    pub tool_input: HashMap<String, Value>,
}

/// Input structure for PostToolUse hooks.
///
/// PostToolUse hooks run immediately after a tool completes successfully.
/// They can provide feedback to Claude but cannot prevent the tool from running
/// (since it already ran).
#[derive(Debug, Deserialize)]
pub struct PostToolUseInput {
    /// Unique identifier for the current Claude Code session
    pub session_id: String,
    /// Path to the conversation transcript JSON file
    pub transcript_path: String,
    /// Name of the tool that was called
    pub tool_name: String,
    /// Tool-specific input parameters that were used
    pub tool_input: HashMap<String, Value>,
    /// Tool-specific response data. The exact schema depends on the tool.
    pub tool_response: HashMap<String, Value>,
}

/// Input structure for Notification hooks.
///
/// Notification hooks run when Claude Code sends notifications, allowing
/// you to customize how you receive alerts (e.g., when Claude needs input
/// or permission to run something).
#[derive(Debug, Deserialize)]
pub struct NotificationInput {
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
pub struct StopInput {
    /// Unique identifier for the current Claude Code session
    pub session_id: String,
    /// Path to the conversation transcript JSON file
    pub transcript_path: String,
    /// True when Claude Code is already continuing as a result of a stop hook.
    /// Check this to prevent infinite loops.
    pub stop_hook_active: bool,
}

/// Output structure for PreToolUse hooks.
///
/// Controls whether a tool call proceeds and provides feedback to Claude.
#[derive(Debug, Serialize, Default)]
pub struct PreToolUseOutput {
    /// Whether to approve or block the tool call.
    /// - `Approve`: Bypasses the permission system, reason shown to user but not Claude
    /// - `Block`: Prevents execution, reason shown to Claude
    /// - `None`: Follows existing permission flow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<Decision>,

    /// Explanation for the decision. Usage depends on decision type:
    /// - For `Approve`: Shown to user but not Claude
    /// - For `Block`: Shown to Claude as feedback
    /// - For `None`: Ignored
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Whether Claude should continue after hook execution (default: true).
    /// If false, Claude stops processing. This is different from blocking
    /// a specific tool call.
    #[serde(rename = "continue", skip_serializing_if = "Option::is_none")]
    pub continue_: Option<bool>,

    /// Message shown to user when continue is false. Not shown to Claude.
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,

    /// Hide output from transcript mode (default: false)
    #[serde(rename = "suppressOutput", skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
}

impl PreToolUseOutput {
    /// Create an approval response
    pub fn approve(reason: &str) -> Self {
        Self {
            decision: Some(Decision::Approve),
            reason: Some(reason.to_string()),
            ..Default::default()
        }
    }

    /// Create a block response
    pub fn block(reason: &str) -> Self {
        Self {
            decision: Some(Decision::Block),
            reason: Some(reason.to_string()),
            ..Default::default()
        }
    }
}

/// Output structure for PostToolUse hooks.
///
/// Provides feedback to Claude after a tool has already executed.
#[derive(Debug, Serialize, Default)]
pub struct PostToolUseOutput {
    /// Whether to provide feedback to Claude.
    /// - `Block`: Automatically prompts Claude with reason
    /// - `None`: No feedback provided
    /// Note: Cannot use `Approve` since the tool already ran
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<Decision>,

    /// Feedback message for Claude (used when decision is Block)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Whether Claude should continue after hook execution (default: true)
    #[serde(rename = "continue", skip_serializing_if = "Option::is_none")]
    pub continue_: Option<bool>,

    /// Message shown to user when continue is false
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,

    /// Hide output from transcript mode (default: false)
    #[serde(rename = "suppressOutput", skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
}

impl PostToolUseOutput {
    /// Create a block response (PostToolUse can only block, not approve)
    pub fn block(reason: &str) -> Self {
        Self {
            decision: Some(Decision::Block),
            reason: Some(reason.to_string()),
            ..Default::default()
        }
    }
}

/// Output structure for Notification hooks.
///
/// Controls continuation and output visibility for notification handling.
#[derive(Debug, Serialize, Default)]
pub struct NotificationOutput {
    /// Whether Claude should continue after hook execution (default: true)
    #[serde(rename = "continue", skip_serializing_if = "Option::is_none")]
    pub continue_: Option<bool>,

    /// Message shown to user when continue is false
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,

    /// Hide output from transcript mode (default: false)
    #[serde(rename = "suppressOutput", skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
}

/// Output structure for Stop hooks.
///
/// Controls whether Claude can stop or must continue processing.
#[derive(Debug, Serialize, Default)]
pub struct StopOutput {
    /// Whether to block Claude from stopping.
    /// - `Block`: Prevents stopping, must provide reason
    /// - `None`: Allows Claude to stop normally
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<Decision>,

    /// Required when decision is Block. Tells Claude how to proceed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// Whether Claude should continue after hook execution (default: true).
    /// Takes precedence over decision if set to false.
    #[serde(rename = "continue", skip_serializing_if = "Option::is_none")]
    pub continue_: Option<bool>,

    /// Message shown to user when continue is false
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,

    /// Hide output from transcript mode (default: false)
    #[serde(rename = "suppressOutput", skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
}

impl StopOutput {
    /// Create a block response to prevent Claude from stopping
    pub fn block(reason: &str) -> Self {
        Self {
            decision: Some(Decision::Block),
            reason: Some(reason.to_string()),
            ..Default::default()
        }
    }
}

/// Helper functions for exit codes.
///
/// Hooks can communicate status through exit codes as an alternative
/// to JSON output:
/// - Exit code 0: Success, stdout shown in transcript mode
/// - Exit code 2: Blocking error, stderr shown to Claude
/// - Other codes: Non-blocking error, stderr shown to user
pub mod exit {
    /// Exit with success (0).
    ///
    /// Stdout will be shown to the user in transcript mode (Ctrl-R).
    pub fn success() {
        std::process::exit(0);
    }

    /// Exit with blocking error (2).
    ///
    /// Stderr will be fed back to Claude to process automatically.
    /// - PreToolUse: Blocks the tool call
    /// - PostToolUse: Shows error to Claude (tool already ran)
    /// - Stop: Blocks stoppage
    pub fn block() {
        std::process::exit(2);
    }

    /// Exit with non-blocking error.
    ///
    /// Stderr is shown to the user and execution continues.
    ///
    /// # Panics
    ///
    /// Panics if code is 0 or 2 (reserved exit codes).
    pub fn error(code: i32) {
        if code == 0 || code == 2 {
            panic!("Invalid error code: {code} (reserved)");
        }
        std::process::exit(code);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_tool_use_output_approve() {
        let output = PreToolUseOutput::approve("Test approval");
        assert!(matches!(output.decision, Some(Decision::Approve)));
        assert_eq!(output.reason, Some("Test approval".to_string()));
    }

    #[test]
    fn test_pre_tool_use_output_block() {
        let output = PreToolUseOutput::block("Test block");
        assert!(matches!(output.decision, Some(Decision::Block)));
        assert_eq!(output.reason, Some("Test block".to_string()));
    }

    #[test]
    fn test_serialization_skips_none() {
        let output = PreToolUseOutput::default();
        let json = serde_json::to_string(&output).unwrap();
        assert_eq!(json, "{}");
    }

    #[test]
    fn test_serialization_with_values() {
        let output = PreToolUseOutput {
            decision: Some(Decision::Approve),
            reason: Some("test".to_string()),
            suppress_output: Some(true),
            ..Default::default()
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"decision\":\"approve\""));
        assert!(json.contains("\"reason\":\"test\""));
        assert!(json.contains("\"suppressOutput\":true"));
    }
}
