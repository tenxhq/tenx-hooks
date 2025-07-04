use serde::{Deserialize, Serialize};

/// Helper function for serde to skip serializing None values
pub(crate) fn is_none<T>(opt: &Option<T>) -> bool {
    opt.is_none()
}

/// Decision type for approve/block operations.
///
/// Used in PreToolUse, PostToolUse, and Stop hooks to control execution flow.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    /// Approve the operation (PreToolUse only - bypasses permission system)
    Approve,
    /// Block the operation and provide feedback to Claude
    Block,
}

/// Output structure for PostToolUse hooks.
///
/// Provides feedback to Claude after a tool has already executed.
#[derive(Debug, Serialize, Default)]
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
    /// Create a block response (PostToolUse can only block, not approve)
    pub fn block(reason: &str) -> Self {
        Self {
            decision: Some(Decision::Block),
            reason: Some(reason.to_string()),
            ..Default::default()
        }
    }

    /// Set the continue field to false and provide a stop reason
    pub fn stop(mut self, reason: &str) -> Self {
        self.continue_ = Some(false);
        self.stop_reason = Some(reason.to_string());
        self
    }

    /// Set whether to suppress output in transcript mode
    pub fn suppress_output(mut self, suppress: bool) -> Self {
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

/// Output structure for Notification hooks.
///
/// Controls continuation and output visibility for notification handling.
#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NotificationOutput {
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

impl NotificationOutput {
    /// Set the continue field to false and provide a stop reason
    pub fn stop(mut self, reason: &str) -> Self {
        self.continue_ = Some(false);
        self.stop_reason = Some(reason.to_string());
        self
    }

    /// Set whether to suppress output in transcript mode
    pub fn suppress_output(mut self, suppress: bool) -> Self {
        self.suppress_output = Some(suppress);
        self
    }
}

/// Output structure for Stop hooks.
///
/// Controls whether Claude can stop or must continue processing.
#[derive(Debug, Serialize, Default)]
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
    pub fn stop(mut self, reason: &str) -> Self {
        self.continue_ = Some(false);
        self.stop_reason = Some(reason.to_string());
        self
    }

    /// Set whether to suppress output in transcript mode
    pub fn suppress_output(mut self, suppress: bool) -> Self {
        self.suppress_output = Some(suppress);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_post_tool_use_output_block() {
        let output = PostToolUseOutput::block("Test block");
        assert!(matches!(output.decision, Some(Decision::Block)));
        assert_eq!(output.reason, Some("Test block".to_string()));
    }

    #[test]
    fn test_stop_output_block() {
        let output = StopOutput::block("Must continue");
        assert!(matches!(output.decision, Some(Decision::Block)));
        assert_eq!(output.reason, Some("Must continue".to_string()));
    }

    #[test]
    fn test_notification_output_builder() {
        let output = NotificationOutput::default()
            .stop("error occurred")
            .suppress_output(true);

        assert_eq!(output.continue_, Some(false));
        assert_eq!(output.stop_reason, Some("error occurred".to_string()));
        assert_eq!(output.suppress_output, Some(true));
    }

    #[test]
    fn test_post_tool_use_builder() {
        let output = PostToolUseOutput::default()
            .stop("tests failed")
            .suppress_output(false);

        assert_eq!(output.decision, None);
        assert_eq!(output.continue_, Some(false));
        assert_eq!(output.stop_reason, Some("tests failed".to_string()));
        assert_eq!(output.suppress_output, Some(false));
    }

    #[test]
    fn test_stop_output_builder() {
        let output = StopOutput::default()
            .stop("must exit")
            .suppress_output(true);

        assert_eq!(output.decision, None);
        assert_eq!(output.continue_, Some(false));
        assert_eq!(output.stop_reason, Some("must exit".to_string()));
        assert_eq!(output.suppress_output, Some(true));
    }
}
