use tenx_hooks::{Hook, Result, output::PreToolUseOutput};

fn main() -> Result<()> {
    let hook = Hook::new();
    let input = hook.pre_tooluse()?;

    // Log some info to stderr for debugging (won't interfere with JSON output)
    eprintln!("Hook received tool: {}", input.tool_name);
    eprintln!("Session ID: {}", input.session_id);

    let approval = PreToolUseOutput::approve("Command looks safe");
    hook.respond(approval)?;

    Ok(())
}
