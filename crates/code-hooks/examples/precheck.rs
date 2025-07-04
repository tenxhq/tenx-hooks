use code_hooks::{HookResponse, Input, PreToolUse, Result};

fn main() -> Result<()> {
    let input = PreToolUse::read()?;

    // Log some info to stderr for debugging (won't interfere with JSON output)
    eprintln!("Hook received tool: {}", input.tool_name);
    eprintln!("Session ID: {}", input.session_id);

    input.approve("Command looks safe").respond();
}
