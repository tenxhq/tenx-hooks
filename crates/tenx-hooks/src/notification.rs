use serde::{Deserialize, Serialize};

use crate::io::{HookResponse, Input, is_none};

/// Input structure for Notification hooks.
///
/// Notification hooks run when Claude Code sends notifications, allowing
/// you to customize how you receive alerts (e.g., when Claude needs input
/// or permission to run something).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Notification {
    /// Unique identifier for the current Claude Code session
    pub session_id: String,

    /// Path to the conversation transcript JSON file
    pub transcript_path: String,

    /// The notification message content
    pub message: String,

    /// The notification title (typically "Claude Code")
    pub hook_event_name: String,
}

impl Notification {
    /// Create a response that allows normal notification handling
    ///
    /// The notification is displayed normally.
    pub fn passthrough() -> NotificationOutput {
        NotificationOutput::default()
    }

    /// Create a response that stops Claude from continuing
    ///
    /// This prevents Claude from continuing after the notification.
    pub fn stop(&self, reason: &str) -> NotificationOutput {
        NotificationOutput::default().and_stop(reason)
    }
}

impl Input for Notification {}

/// Output structure for Notification hooks.
///
/// Controls continuation and output visibility for notification handling.
#[derive(Debug, Serialize, Deserialize, Default)]
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

impl HookResponse for NotificationOutput {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_roundtrip() {
        // Create a Notification instance
        let notification = Notification {
            session_id: "test-session".to_string(),
            transcript_path: "/path/to/transcript".to_string(),
            message: "Claude needs permission to run a command".to_string(),
            hook_event_name: "Claude Code".to_string(),
        };

        // Test passthrough response
        let passthrough_output = Notification::passthrough();
        let json = serde_json::to_string(&passthrough_output).unwrap();
        let deserialized: NotificationOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.continue_, passthrough_output.continue_);
        assert_eq!(deserialized.stop_reason, passthrough_output.stop_reason);

        // Test stop response
        let stop_output = notification.stop("User intervention required");
        assert_eq!(stop_output.continue_, Some(false));
        assert_eq!(
            stop_output.stop_reason,
            Some("User intervention required".to_string())
        );
    }
}
