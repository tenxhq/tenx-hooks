//! A Rust library for building hooks for Claude Code.
//!
//! Claude Code hooks are user-defined shell commands that execute at various points
//! in Claude Code's lifecycle. They provide deterministic control over Claude Code's
//! behavior, ensuring certain actions always happen rather than relying on the LLM
//! to choose to run them.
//!
//! This library implements the JSON-based hook protocol used by Claude Code only, avoiding the
//! less well-defined error code protocol. This means code-hooks tools always exit with status code
//! 0, and return well-formed JSON responses.
//!
//! # Example
//!
//! ```rust,no_run
//! use code_hooks::{HookResponse, Input, PreToolUse, PreToolUseOutput, Result};
//!
//! fn main() -> Result<()> {
//!     let input = PreToolUse::read()?;
//!     
//!     if input.tool_name == "Bash" {
//!         if let Some(command) = input.tool_input.get("command").and_then(|v| v.as_str()) {
//!             if command.contains("rm -rf") {
//!                 PreToolUseOutput::block("Dangerous command detected").respond();
//!             }
//!         }
//!     }
//!     
//!     PreToolUseOutput::approve("Command validated").respond();
//! }
//! ```

mod error;
mod io;
mod notification;
mod posttool;
mod pretool;
mod stop;
mod subagent_stop;

pub use error::{Error, Result};
pub use io::{Decision, HookResponse, Input, TranscriptReader};
pub use notification::{Notification, NotificationOutput};
pub use posttool::{PostToolUse, PostToolUseOutput};
pub use pretool::{PreToolUse, PreToolUseOutput};
pub use stop::{Stop, StopOutput};
pub use subagent_stop::{SubagentStop, SubagentStopOutput};
