use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::io::{self, Read};

/// Main hook interface for interacting with Claude Code
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
                println!("{}", json);
            }
            Err(e) => {
                eprintln!("Failed to serialize output: {}", e);
                std::process::exit(1);
            }
        }
    }

    /// Internal method to read and parse JSON from stdin
    fn read_input<T: for<'de> Deserialize<'de>>(&self) -> Result<T, HookError> {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| HookError::IoError(e))?;
        
        serde_json::from_str(&buffer)
            .map_err(|e| HookError::ParseError(e))
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
    IoError(io::Error),
    ParseError(serde_json::Error),
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookError::IoError(e) => write!(f, "IO error: {}", e),
            HookError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for HookError {}

/// Decision type for approve/block operations
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Approve,
    Block,
}

/// Common fields shared by all input types
#[derive(Debug, Deserialize)]
pub struct CommonInput {
    pub session_id: String,
    pub transcript_path: String,
}

/// Input structure for PreToolUse hooks
#[derive(Debug, Deserialize)]
pub struct PreToolUseInput {
    pub session_id: String,
    pub transcript_path: String,
    pub tool_name: String,
    pub tool_input: HashMap<String, Value>,
}

/// Input structure for PostToolUse hooks
#[derive(Debug, Deserialize)]
pub struct PostToolUseInput {
    pub session_id: String,
    pub transcript_path: String,
    pub tool_name: String,
    pub tool_input: HashMap<String, Value>,
    pub tool_response: HashMap<String, Value>,
}

/// Input structure for Notification hooks
#[derive(Debug, Deserialize)]
pub struct NotificationInput {
    pub session_id: String,
    pub transcript_path: String,
    pub message: String,
    pub title: String,
}

/// Input structure for Stop hooks
#[derive(Debug, Deserialize)]
pub struct StopInput {
    pub session_id: String,
    pub transcript_path: String,
    pub stop_hook_active: bool,
}

/// Output structure for PreToolUse hooks
#[derive(Debug, Serialize, Default)]
pub struct PreToolUseOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<Decision>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    
    #[serde(rename = "continue", skip_serializing_if = "Option::is_none")]
    pub continue_: Option<bool>,
    
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    
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

/// Output structure for PostToolUse hooks
#[derive(Debug, Serialize, Default)]
pub struct PostToolUseOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<Decision>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    
    #[serde(rename = "continue", skip_serializing_if = "Option::is_none")]
    pub continue_: Option<bool>,
    
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    
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

/// Output structure for Notification hooks
#[derive(Debug, Serialize, Default)]
pub struct NotificationOutput {
    #[serde(rename = "continue", skip_serializing_if = "Option::is_none")]
    pub continue_: Option<bool>,
    
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    
    #[serde(rename = "suppressOutput", skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
}

/// Output structure for Stop hooks
#[derive(Debug, Serialize, Default)]
pub struct StopOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<Decision>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    
    #[serde(rename = "continue", skip_serializing_if = "Option::is_none")]
    pub continue_: Option<bool>,
    
    #[serde(rename = "stopReason", skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    
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

/// Helper functions for exit codes
pub mod exit {
    /// Exit with success (0)
    pub fn success() {
        std::process::exit(0);
    }

    /// Exit with blocking error (2) - shows stderr to Claude
    pub fn block() {
        std::process::exit(2);
    }

    /// Exit with non-blocking error
    pub fn error(code: i32) {
        if code == 0 || code == 2 {
            panic!("Invalid error code: {} (reserved)", code);
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