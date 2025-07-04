use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::input::Input;
use crate::output::{Decision, is_none};
use crate::response::HookResponse;

/// Input structure for PostToolUse hooks.
///
/// PostToolUse hooks run immediately after a tool completes successfully.
/// They can provide feedback to Claude but cannot prevent the tool from running
/// (since it already ran).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PostToolUse {
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

impl PostToolUse {
    /// Create a block response that suppresses the normal tool result
    ///
    /// Claude sees the reason as an error message instead of the actual tool output.
    pub fn block(&self, reason: &str) -> PostToolUseOutput {
        PostToolUseOutput::block(reason)
    }

    /// Create a passthrough response that sends the normal tool result to Claude
    ///
    /// The tool result is passed through unchanged. Any reason provided would be discarded.
    pub fn passthrough(&self) -> PostToolUseOutput {
        PostToolUseOutput::passthrough()
    }
}

impl Input for PostToolUse {}

/// Output structure for PostToolUse hooks.
///
/// Provides feedback to Claude after a tool has already executed.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PostToolUseOutput {
    /// Whether to provide feedback to Claude.
    /// - `Block`: Automatically prompts Claude with reason
    /// - `None`: No feedback provided
    ///
    /// Note: Cannot use `Approve` since the tool already ran
    #[serde(skip_serializing_if = "is_none")]
    pub decision: Option<Decision>,

    /// Feedback message for Claude (used when decision is Block)
    #[serde(skip_serializing_if = "is_none")]
    pub reason: Option<String>,

    /// Whether Claude should continue after hook execution (default: true)
    #[serde(rename = "continue", skip_serializing_if = "is_none")]
    pub continue_: Option<bool>,

    /// Message shown to user when continue is false
    #[serde(skip_serializing_if = "is_none")]
    pub stop_reason: Option<String>,

    /// Hide output from transcript mode (default: false)
    #[serde(skip_serializing_if = "is_none")]
    pub suppress_output: Option<bool>,
}

impl PostToolUseOutput {
    /// Create a block response that suppresses the normal tool result
    ///
    /// Claude sees the reason as an error message instead of the actual tool output.
    /// The tool has already executed, but this replaces what Claude sees.
    pub fn block(reason: &str) -> Self {
        Self {
            decision: Some(Decision::Block),
            reason: Some(reason.to_string()),
            ..Default::default()
        }
    }

    /// Create a passthrough response that sends the normal tool result to Claude
    ///
    /// This omits the decision field, so the normal tool_result is passed to Claude.
    /// Any reason provided would be discarded since there's no decision to attach it to.
    pub fn passthrough() -> Self {
        Self::default()
    }

    /// Set the continue field to false and provide a stop reason
    pub fn and_stop(mut self, reason: &str) -> Self {
        self.continue_ = Some(false);
        self.stop_reason = Some(reason.to_string());
        self
    }

    /// Set whether to suppress output in transcript mode
    pub fn and_suppress_output(mut self, suppress: bool) -> Self {
        self.suppress_output = Some(suppress);
        self
    }

    /// Set a custom reason (overwrites the one from block)
    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }

    /// Clear the reason
    pub fn without_reason(mut self) -> Self {
        self.reason = None;
        self
    }
}

impl HookResponse for PostToolUseOutput {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_tool_use_roundtrip() {
        // Create a PostToolUse instance
        let mut tool_response = HashMap::new();
        tool_response.insert(
            "output".to_string(),
            Value::String("Command executed".to_string()),
        );

        let post_tool_use = PostToolUse {
            session_id: "test-session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            tool_name: "Bash".to_string(),
            tool_input: HashMap::new(),
            tool_response,
        };

        // Test block response
        let block_output = post_tool_use.block("Found sensitive data");
        let json = serde_json::to_string(&block_output).unwrap();
        let deserialized: PostToolUseOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.decision, block_output.decision);
        assert_eq!(deserialized.reason, block_output.reason);
    }
}
