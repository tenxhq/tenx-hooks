use crate::error::Result;
use serde::Deserialize;
use std::io::{self, Read as IoRead};

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
