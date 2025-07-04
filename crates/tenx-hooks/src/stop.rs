use serde::{Deserialize, Serialize};

use crate::input::Input;
use crate::output::{Decision, is_none};
use crate::response::HookResponse;

/// Input structure for Stop hooks.
///
/// Stop hooks run when Claude Code has finished responding. They can
/// block Claude from stopping and request continuation.
#[derive(Debug, Serialize, Deserialize)]
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

impl Stop {
    /// Create a block response to prevent Claude from stopping
    ///
    /// Claude will continue processing with the provided reason.
    pub fn block(&self, reason: &str) -> StopOutput {
        StopOutput::block(reason)
    }

    /// Create a response that allows Claude to stop normally
    ///
    /// This is the default behavior when no decision is provided.
    pub fn allow(&self) -> StopOutput {
        StopOutput::default()
    }

    /// Create a response that stops Claude immediately
    ///
    /// This prevents any further processing and shows the stop reason to the user.
    pub fn stop(&self, reason: &str) -> StopOutput {
        StopOutput::default().and_stop(reason)
    }
}

impl Input for Stop {}

/// Output structure for Stop hooks.
///
/// Controls whether Claude can stop or must continue processing.
#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StopOutput {
    /// Whether to block Claude from stopping.
    /// - `Block`: Prevents stopping, must provide reason
    /// - `None`: Allows Claude to stop normally
    #[serde(skip_serializing_if = "is_none")]
    pub decision: Option<Decision>,

    /// Required when decision is Block. Tells Claude how to proceed.
    #[serde(skip_serializing_if = "is_none")]
    pub reason: Option<String>,

    /// Whether Claude should continue after hook execution (default: true).
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

impl StopOutput {
    /// Create a block response to prevent Claude from stopping
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

impl HookResponse for StopOutput {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stop_roundtrip() {
        // Create a Stop instance
        let stop = Stop {
            session_id: "test-session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            stop_hook_active: false,
        };

        // Test block response
        let block_output = stop.block("Need to process more");
        let json = serde_json::to_string(&block_output).unwrap();
        let deserialized: StopOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.decision, Some(Decision::Block));
        assert_eq!(
            deserialized.reason,
            Some("Need to process more".to_string())
        );

        // Test allow response
        let allow_output = stop.allow();
        assert_eq!(allow_output.decision, None);
        assert_eq!(allow_output.continue_, None);

        // Test stop response
        let stop_output = stop.stop("Task completed");
        assert_eq!(stop_output.continue_, Some(false));
        assert_eq!(stop_output.stop_reason, Some("Task completed".to_string()));
    }
}
