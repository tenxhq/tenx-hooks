use crate::error::Result;
use crate::input::{Notification, PostToolUse, Stop};
use crate::pretool::PreToolUse;
use serde::{Deserialize, Serialize};
use std::io::{self, Read};

/// Main hook interface for interacting with Claude Code.
///
/// The `Hook` struct provides methods to read input from stdin and send
/// responses to stdout, handling all JSON serialization/deserialization
/// automatically.
pub struct Hook;

impl Hook {
    /// Create a new Hook instance
    pub fn new() -> Self {
        Hook
    }

    /// Read PreToolUse from stdin
    pub fn pre_tooluse(&self) -> Result<PreToolUse> {
        self.read_input()
    }

    /// Read PostToolUse input from stdin
    pub fn post_tooluse(&self) -> Result<PostToolUse> {
        self.read_input()
    }

    /// Read Notification input from stdin
    pub fn notification(&self) -> Result<Notification> {
        self.read_input()
    }

    /// Read Stop input from stdin
    pub fn stop(&self) -> Result<Stop> {
        self.read_input()
    }

    /// Send a response to stdout
    pub fn respond<T: Serialize>(&self, output: T) -> Result<()> {
        let json = serde_json::to_string(&output)?;
        println!("{json}");
        Ok(())
    }

    /// Internal method to read and parse JSON from stdin
    fn read_input<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        let parsed = serde_json::from_str(&buffer)?;
        Ok(parsed)
    }
}

impl Default for Hook {
    fn default() -> Self {
        Self::new()
    }
}
