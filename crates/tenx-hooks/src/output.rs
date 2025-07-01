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

/// Output structure for PreToolUse hooks.
///
/// Controls whether a tool call proceeds and provides feedback to Claude.
#[derive(Debug, Serialize, Default)]
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

    /// Set a custom reason (overwrites the one from approve/block)
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

    #[test]
    fn test_camel_case_serialization() {
        let output = PreToolUseOutput {
            stop_reason: Some("test stop".to_string()),
            suppress_output: Some(true),
            ..Default::default()
        };
        let json = serde_json::to_string(&output).unwrap();
        assert!(json.contains("\"stopReason\":\"test stop\""));
        assert!(json.contains("\"suppressOutput\":true"));
        assert!(!json.contains("stop_reason"));
        assert!(!json.contains("suppress_output"));
    }

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
    fn test_builder_chaining() {
        // Test PreToolUseOutput chaining
        let output = PreToolUseOutput::block("error")
            .stop("fatal error")
            .suppress_output(true);

        assert!(matches!(output.decision, Some(Decision::Block)));
        assert_eq!(output.reason, Some("error".to_string()));
        assert_eq!(output.continue_, Some(false));
        assert_eq!(output.stop_reason, Some("fatal error".to_string()));
        assert_eq!(output.suppress_output, Some(true));

        // Test with_reason overwrites original reason
        let output = PreToolUseOutput::approve("ok").with_reason("different reason");
        assert_eq!(output.reason, Some("different reason".to_string()));

        // Test without_reason clears reason
        let output = PreToolUseOutput::approve("ok").without_reason();
        assert_eq!(output.reason, None);
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
