//! A Rust library for building hooks for Claude Code.
//!
//! Claude Code hooks are user-defined shell commands that execute at various points
//! in Claude Code's lifecycle. They provide deterministic control over Claude Code's
//! behavior, ensuring certain actions always happen rather than relying on the LLM
//! to choose to run them.
//!
//! # Example
//!
//! ```rust,no_run
//! use tenx_hooks::{Hook, output::PreToolUseOutput, Result};
//!
//! fn main() -> Result<()> {
//!     let hook = Hook::new();
//!     let input = hook.pre_tooluse()?;
//!     
//!     if input.tool_name == "Bash" {
//!         if let Some(command) = input.tool_input.get("command").and_then(|v| v.as_str()) {
//!             if command.contains("rm -rf") {
//!                 hook.respond(PreToolUseOutput::block("Dangerous command detected"))?;
//!                 return Ok(());
//!             }
//!         }
//!     }
//!     
//!     hook.respond(PreToolUseOutput::approve("Command validated"))?;
//!     Ok(())
//! }
//! ```

mod error;
mod hook;
mod input;

pub mod exit;
pub mod output;

pub use error::{Error, Result};
pub use hook::Hook;
pub use input::*;
