use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::input::Input;
use crate::output::{Decision, is_none};
use crate::response::HookResponse;

/// Input structure for PreToolUse hooks.
///
/// PreToolUse hooks run after Claude creates tool parameters but before
/// processing the tool call. They can approve or block the operation.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PreToolUse {
    /// Unique identifier for the current Claude Code session
    pub session_id: String,
    /// Path to the conversation transcript JSON file
    pub transcript_path: String,
    /// Name of the tool being called (e.g., "Bash", "Write", "Edit")
    pub tool_name: String,
    /// Tool-specific input parameters. The exact schema depends on the tool.
    pub tool_input: HashMap<String, Value>,
}

impl PreToolUse {
    /// Create an approval response that bypasses the permission system
    ///
    /// The tool executes immediately. The reason is shown to the user but not Claude.
    pub fn approve(&self, reason: &str) -> PreToolUseOutput {
        PreToolUseOutput::approve(reason)
    }

    /// Create a block response that prevents tool execution
    ///
    /// The reason is passed to Claude as feedback.
    pub fn block(&self, reason: &str) -> PreToolUseOutput {
        PreToolUseOutput::block(reason)
    }

    /// Create a passthrough response that defers to Claude's regular approval flow
    ///
    /// The agent may show an approval dialogue or proceed based on its configuration.
    pub fn passthrough(&self) -> PreToolUseOutput {
        PreToolUseOutput::passthrough()
    }
}

impl Input for PreToolUse {}

/// Output structure for PreToolUse hooks.
///
/// Controls whether a tool call proceeds and provides feedback to Claude.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PreToolUseOutput {
    /// Whether to approve or block the tool call.
    /// - `Approve`: Bypasses the permission system, reason shown to user but not Claude
    /// - `Block`: Prevents execution, reason shown to Claude
    /// - `None`: Follows existing permission flow
    #[serde(skip_serializing_if = "is_none")]
    pub decision: Option<Decision>,

    /// Explanation for the decision. Usage depends on decision type:
    /// - For `Approve`: Shown to user but not Claude
    /// - For `Block`: Shown to Claude as feedback
    /// - For `None`: Ignored
    #[serde(skip_serializing_if = "is_none")]
    pub reason: Option<String>,

    /// Whether Claude should continue after hook execution (default: true).
    /// If false, Claude stops processing. This is different from blocking
    /// a specific tool call.
    #[serde(rename = "continue", skip_serializing_if = "is_none")]
    pub continue_: Option<bool>,

    /// Message shown to user when continue is false. Not shown to Claude.
    #[serde(skip_serializing_if = "is_none")]
    pub stop_reason: Option<String>,

    /// Hide output from transcript mode (default: false)
    #[serde(skip_serializing_if = "is_none")]
    pub suppress_output: Option<bool>,
}

impl PreToolUseOutput {
    /// Create an approval response that bypasses the permission system
    ///
    /// The tool executes immediately. The reason is shown to the user but not Claude.
    pub fn approve(reason: &str) -> Self {
        Self {
            decision: Some(Decision::Approve),
            reason: Some(reason.to_string()),
            ..Default::default()
        }
    }

    /// Create a block response that prevents tool execution
    ///
    /// The reason is passed to Claude as feedback.
    pub fn block(reason: &str) -> Self {
        Self {
            decision: Some(Decision::Block),
            reason: Some(reason.to_string()),
            ..Default::default()
        }
    }

    /// Create a passthrough response that defers to Claude's regular approval flow
    ///
    /// This omits the decision field, allowing the agent to show an approval
    /// dialogue or proceed based on its configuration.
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
}

impl HookResponse for PreToolUseOutput {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pre_tool_use_roundtrip() {
        // Create a PreToolUse instance
        let pre_tool_use = PreToolUse {
            session_id: "test-session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            tool_name: "Bash".to_string(),
            tool_input: HashMap::new(),
        };

        // Test approve response
        let approve_output = pre_tool_use.approve("Approved for testing");
        let json = serde_json::to_string(&approve_output).unwrap();
        let deserialized: PreToolUseOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.decision, approve_output.decision);
        assert_eq!(deserialized.reason, approve_output.reason);
    }
}
