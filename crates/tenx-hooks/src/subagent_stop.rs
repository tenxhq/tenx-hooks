use serde::{Deserialize, Serialize};

use crate::io::{Decision, HookResponse, Input, is_none};

/// Input structure for SubagentStop hooks.
///
/// SubagentStop hooks run when a subagent has finished responding. They can
/// block the subagent from stopping and request continuation. This event has
/// exactly the same semantics as Stop but is only called for subagents.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SubagentStop {
    /// Unique identifier for the current Claude Code session
    pub session_id: String,
    /// Path to the conversation transcript JSON file
    pub transcript_path: String,
    /// True when the subagent is already continuing as a result of a SubagentStop hook.
    /// Check this to prevent infinite loops.
    pub stop_hook_active: bool,
}

impl SubagentStop {
    /// Create a block response to prevent the subagent from stopping
    ///
    /// The subagent will continue processing with the provided reason.
    pub fn block(&self, reason: &str) -> SubagentStopOutput {
        SubagentStopOutput::block(reason)
    }

    /// Create a response that allows the subagent to stop normally
    ///
    /// This is the default behavior when no decision is provided.
    pub fn allow(&self) -> SubagentStopOutput {
        SubagentStopOutput::default()
    }

    /// Create a response that stops the subagent immediately
    ///
    /// This prevents any further processing and shows the stop reason to the user.
    pub fn stop(&self, reason: &str) -> SubagentStopOutput {
        SubagentStopOutput::default().and_stop(reason)
    }
}

impl Input for SubagentStop {}

/// Output structure for SubagentStop hooks.
///
/// Controls whether the subagent can stop or must continue processing.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SubagentStopOutput {
    /// Whether to block the subagent from stopping.
    /// - `Block`: Prevents stopping, must provide reason
    /// - `None`: Allows the subagent to stop normally
    #[serde(skip_serializing_if = "is_none")]
    pub decision: Option<Decision>,

    /// Required when decision is Block. Tells the subagent how to proceed.
    #[serde(skip_serializing_if = "is_none")]
    pub reason: Option<String>,

    /// Whether the subagent should continue after hook execution (default: true).
    /// Takes precedence over decision if set to false.
    #[serde(rename = "continue", skip_serializing_if = "is_none")]
    pub continue_: Option<bool>,

    /// Message shown to user when continue is false
    #[serde(skip_serializing_if = "is_none")]
    pub stop_reason: Option<String>,

    /// Hide output from transcript mode (default: false)
    #[serde(skip_serializing_if = "is_none")]
    pub suppress_output: Option<bool>,
}

impl SubagentStopOutput {
    /// Create a block response to prevent the subagent from stopping
    pub fn block(reason: &str) -> Self {
        Self {
            decision: Some(Decision::Block),
            reason: Some(reason.to_string()),
            ..Default::default()
        }
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

impl HookResponse for SubagentStopOutput {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subagent_stop_roundtrip() {
        // Create a SubagentStop instance
        let subagent_stop = SubagentStop {
            session_id: "test-session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            stop_hook_active: false,
        };

        // Test block response
        let block_output = subagent_stop.block("Need to process more");
        let json = serde_json::to_string(&block_output).unwrap();
        let deserialized: SubagentStopOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.decision, Some(Decision::Block));
        assert_eq!(
            deserialized.reason,
            Some("Need to process more".to_string())
        );

        // Test allow response
        let allow_output = subagent_stop.allow();
        assert_eq!(allow_output.decision, None);
        assert_eq!(allow_output.continue_, None);

        // Test stop response
        let stop_output = subagent_stop.stop("Task completed");
        assert_eq!(stop_output.continue_, Some(false));
        assert_eq!(stop_output.stop_reason, Some("Task completed".to_string()));
    }
}
