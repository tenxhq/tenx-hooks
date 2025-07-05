pub mod parse;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Main enum that represents different types of transcript entries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TranscriptEntry {
    User(UserEntry),
    Assistant(AssistantEntry),
    Summary(SummaryEntry),
    System(SystemEntry),
}

/// User message entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserEntry {
    pub uuid: String,
    pub timestamp: String,
    pub message: TranscriptMessage,
    pub cwd: String,
    pub session_id: String,
    pub version: String,
    pub user_type: String,
    pub is_sidechain: bool,
    pub parent_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_result: Option<Value>,
}

/// Assistant message entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantEntry {
    pub uuid: String,
    pub timestamp: String,
    pub message: TranscriptMessage,
    pub cwd: String,
    pub session_id: String,
    pub version: String,
    pub user_type: String,
    pub is_sidechain: bool,
    pub parent_uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_api_error_message: Option<bool>,
}

/// Summary entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryEntry {
    pub summary: String,
    pub leaf_uuid: String,
}

/// System message entry
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemEntry {
    pub uuid: String,
    pub timestamp: String,
    pub content: String,
    pub cwd: String,
    pub session_id: String,
    pub version: String,
    pub user_type: String,
    pub is_sidechain: bool,
    pub parent_uuid: String,
    pub is_meta: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(rename = "toolUseID", skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
}

/// Message can be either from a user or an assistant
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum TranscriptMessage {
    User {
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<MessageContent>,
    },
    Assistant {
        id: String,
        #[serde(rename = "type")]
        message_type: String,
        model: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        content: Option<MessageContent>,
        #[serde(skip_serializing_if = "Option::is_none")]
        thinking: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_uses: Option<Vec<ToolUse>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        code_outputs: Option<Vec<CodeOutput>>,
        stop_reason: Option<String>,
        stop_sequence: Option<String>,
        usage: UsageInfo,
    },
}

impl TranscriptMessage {
    /// Get the content of the message regardless of type
    pub fn content(&self) -> Option<&MessageContent> {
        match self {
            TranscriptMessage::User { content } => content.as_ref(),
            TranscriptMessage::Assistant { content, .. } => content.as_ref(),
        }
    }
}

/// Content can be either a simple string or an array of content blocks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

impl MessageContent {
    /// Check if content contains tool uses
    pub fn has_tool_uses(&self) -> bool {
        match self {
            MessageContent::Text(_) => false,
            MessageContent::Blocks(blocks) => blocks
                .iter()
                .any(|b| matches!(b, ContentBlock::ToolUse { .. })),
        }
    }

    /// Count tool uses in content
    pub fn count_tool_uses(&self) -> usize {
        match self {
            MessageContent::Text(_) => 0,
            MessageContent::Blocks(blocks) => blocks
                .iter()
                .filter(|b| matches!(b, ContentBlock::ToolUse { .. }))
                .count(),
        }
    }

    /// Count tool results in content
    pub fn count_tool_results(&self) -> usize {
        match self {
            MessageContent::Text(_) => 0,
            MessageContent::Blocks(blocks) => blocks
                .iter()
                .filter(|b| matches!(b, ContentBlock::ToolResult { .. }))
                .count(),
        }
    }
}

/// Tool result content can be either a string or an array of content items
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    Text(String),
    Array(Vec<ToolResultItem>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultItem {
    #[serde(rename = "type")]
    pub item_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        tool_use_id: String,
        content: ToolResultContent,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
    Thinking {
        thinking: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolUse {
    pub tool_name: String,
    pub tool_input: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_output: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeOutput {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
}
