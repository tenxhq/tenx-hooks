use crate::TranscriptEntry;
use serde_json;
use std::fmt;

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
