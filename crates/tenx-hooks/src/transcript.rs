use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptEntry {
    #[serde(rename = "type")]
    pub entry_type: TranscriptEntryType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<TranscriptMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_usage: Option<TokenUsage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TranscriptEntryType {
    System,
    User,
    Assistant,
    Result,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<MessageContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_uses: Option<Vec<ToolUse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_outputs: Option<Vec<CodeOutput>>,
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
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u64>,
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

impl TranscriptEntry {
    /// Format the entry as pretty-printed JSON for debugging
    pub fn to_debug_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Get a brief description of the entry for logging
    pub fn description(&self) -> String {
        match &self.entry_type {
            TranscriptEntryType::System => {
                format!("System: {}", self.subtype.as_deref().unwrap_or("init"))
            }
            TranscriptEntryType::User => {
                let preview = self
                    .message
                    .as_ref()
                    .and_then(|m| m.content.as_ref())
                    .map(|c| {
                        let text = c.as_text();
                        let truncated = text.chars().take(50).collect::<String>();
                        if text.len() > 50 {
                            format!("{truncated}...")
                        } else {
                            truncated
                        }
                    })
                    .unwrap_or_else(|| "No content".to_string());
                format!("User: {preview}")
            }
            TranscriptEntryType::Assistant => {
                let has_thinking = self
                    .message
                    .as_ref()
                    .and_then(|m| m.thinking.as_ref())
                    .is_some();

                // Count tool uses from both tool_uses field and content
                let tool_uses_count = self
                    .message
                    .as_ref()
                    .and_then(|m| m.tool_uses.as_ref())
                    .map(|t| t.len())
                    .unwrap_or(0);

                let content_tool_count = self
                    .message
                    .as_ref()
                    .and_then(|m| m.content.as_ref())
                    .map(|c| c.count_tool_uses())
                    .unwrap_or(0);

                let total_tool_count = tool_uses_count + content_tool_count;

                let code_count = self
                    .message
                    .as_ref()
                    .and_then(|m| m.code_outputs.as_ref())
                    .map(|c| c.len())
                    .unwrap_or(0);

                let mut parts = vec!["Assistant".to_string()];
                if has_thinking {
                    parts.push("with thinking".to_string());
                }
                if total_tool_count > 0 {
                    parts.push(format!("{total_tool_count} tool uses"));
                }
                if code_count > 0 {
                    parts.push(format!("{code_count} code outputs"));
                }
                parts.join(": ")
            }
            TranscriptEntryType::Result => {
                format!("Result: {}", self.status.as_deref().unwrap_or("unknown"))
            }
            TranscriptEntryType::Summary => "Summary".to_string(),
        }
    }
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

/// Format a raw JSON line for debugging display
pub fn format_json_line_for_debug(line: &str) -> String {
    // Try to parse and pretty-print the JSON
    if let Ok(value) = serde_json::from_str::<Value>(line) {
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| line.to_string())
    } else {
        // If it's not valid JSON, return as-is
        line.to_string()
    }
}
