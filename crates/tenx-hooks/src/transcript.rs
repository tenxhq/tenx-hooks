use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

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
    pub request_id: String,
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
    pub level: String,
    #[serde(rename = "toolUseID")]
    pub tool_use_id: String,
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

    /// Get the role as a string
    pub fn role(&self) -> &str {
        match self {
            TranscriptMessage::User { .. } => "user",
            TranscriptMessage::Assistant { .. } => "assistant",
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
    /// Get the text content as a single string
    pub fn as_text(&self) -> String {
        match self {
            MessageContent::Text(text) => text.clone(),
            MessageContent::Blocks(blocks) => blocks
                .iter()
                .map(|b| match b {
                    ContentBlock::Text { text, .. } => text.clone(),
                    ContentBlock::ToolUse { name, .. } => name.clone(),
                    ContentBlock::ToolResult { content, .. } => match content {
                        ToolResultContent::Text(text) => text.clone(),
                        ToolResultContent::Array(items) => items
                            .iter()
                            .map(|item| item.text.as_str())
                            .collect::<Vec<_>>()
                            .join(" "),
                    },
                })
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

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

impl TranscriptEntry {
    /// Get the UUID if available
    pub fn uuid(&self) -> Option<&str> {
        match self {
            TranscriptEntry::User(entry) => Some(&entry.uuid),
            TranscriptEntry::Assistant(entry) => Some(&entry.uuid),
            TranscriptEntry::System(entry) => Some(&entry.uuid),
            TranscriptEntry::Summary(_) => None,
        }
    }

    /// Get the timestamp if available
    pub fn timestamp(&self) -> Option<&str> {
        match self {
            TranscriptEntry::User(entry) => Some(&entry.timestamp),
            TranscriptEntry::Assistant(entry) => Some(&entry.timestamp),
            TranscriptEntry::System(entry) => Some(&entry.timestamp),
            TranscriptEntry::Summary(_) => None,
        }
    }
}

/// Error type for transcript parsing with detailed context
#[derive(Debug)]
pub struct TranscriptParseError {
    /// Line number where the error occurred (1-indexed)
    pub line_number: usize,
    /// The raw line content that failed to parse
    pub line_content: String,
    /// The underlying JSON parsing error
    pub json_error: serde_json::Error,
}

impl fmt::Display for TranscriptParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed to parse transcript at line {}: {}",
            self.line_number, self.json_error
        )
    }
}

impl std::error::Error for TranscriptParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.json_error)
    }
}

/// Result of parsing a transcript with detailed information
#[derive(Debug)]
pub struct TranscriptParseResult {
    /// Successfully parsed entries
    pub entries: Vec<TranscriptEntry>,
    /// Errors encountered during parsing (if any)
    pub errors: Vec<TranscriptParseError>,
}

pub fn parse_transcript_line(line: &str) -> Result<TranscriptEntry, serde_json::Error> {
    serde_json::from_str(line)
}

pub fn parse_transcript(content: &str) -> Result<Vec<TranscriptEntry>, serde_json::Error> {
    content
        .lines()
        .filter(|line| !line.is_empty())
        .map(parse_transcript_line)
        .collect()
}

/// Parse a transcript with detailed error context for debugging
pub fn parse_transcript_with_context(content: &str) -> TranscriptParseResult {
    let mut entries = Vec::new();
    let mut errors = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        if line.is_empty() {
            continue;
        }

        match parse_transcript_line(line) {
            Ok(entry) => entries.push(entry),
            Err(json_error) => {
                errors.push(TranscriptParseError {
                    line_number: line_idx + 1,
                    line_content: line.to_string(),
                    json_error,
                });
            }
        }
    }

    TranscriptParseResult { entries, errors }
}
