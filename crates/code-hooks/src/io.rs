use crate::error::Result;
use crate::transcript::TranscriptEntry;
use serde::{Deserialize, Serialize};
use std::io::{self, Read};
use std::process;

/// Trait for hook input types that can be read from stdin.
///
/// This trait provides a standard way to read hook inputs by:
/// 1. Reading all content from stdin
/// 2. Parsing it as JSON
/// 3. Deserializing to the appropriate type
///
/// # Example
///
/// ```rust,no_run
/// use tenx_hooks::{Input, PreToolUse};
///
/// let input = PreToolUse::read().expect("Failed to read input");
/// println!("Tool name: {}", input.tool_name);
/// ```
pub trait Input: for<'de> Deserialize<'de> + Sized {
    /// Read and parse input from stdin.
    fn read() -> Result<Self> {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        let parsed = serde_json::from_str(&buffer)?;
        Ok(parsed)
    }
}

/// Trait for hook response types that can be serialized and sent to stdout.
///
/// This trait provides a standard way to respond from Claude Code hooks by:
/// 1. Serializing the response to JSON
/// 2. Printing it to stdout
/// 3. Exiting with status code 0
///
/// # Example
///
/// ```no_run
/// use tenx_hooks::{HookResponse, PreToolUseOutput};
///
/// let response = PreToolUseOutput::approve("Looks good");
/// response.respond(); // Prints JSON and exits
/// ```
pub trait HookResponse: Serialize {
    /// Serialize the response to JSON, print to stdout, and exit with status 0.
    fn respond(self) -> !
    where
        Self: Sized,
    {
        match serde_json::to_string(&self) {
            Ok(json) => {
                println!("{json}");
                process::exit(0);
            }
            Err(e) => {
                eprintln!("Failed to serialize response: {e}");
                process::exit(1);
            }
        }
    }
}

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

/// Trait for hook input types that can read their associated transcript file.
///
/// This trait provides a standard way to read and parse the transcript file
/// referenced in the hook input's transcript_path field.
pub trait TranscriptReader {
    /// Read and parse the transcript file.
    ///
    /// Returns a vector of transcript entries from the JSONL file at transcript_path.
    fn read_transcript(&self) -> Result<Vec<TranscriptEntry>>;
}
