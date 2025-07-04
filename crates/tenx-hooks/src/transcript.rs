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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_uses: Option<Vec<ToolUse>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code_outputs: Option<Vec<CodeOutput>>,
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
                        let truncated = c.chars().take(50).collect::<String>();
                        if c.len() > 50 {
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
                let tool_count = self
                    .message
                    .as_ref()
                    .and_then(|m| m.tool_uses.as_ref())
                    .map(|t| t.len())
                    .unwrap_or(0);
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
                if tool_count > 0 {
                    parts.push(format!("{tool_count} tool uses"));
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
