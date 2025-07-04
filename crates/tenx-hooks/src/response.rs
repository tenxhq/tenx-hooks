use serde::Serialize;
use std::process;

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
