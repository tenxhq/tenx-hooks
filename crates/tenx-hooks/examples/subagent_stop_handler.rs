use tenx_hooks::{HookResponse, Input, Result, SubagentStop};

fn main() -> Result<()> {
    // Read the hook input from stdin
    let subagent_stop = SubagentStop::read()?;

    // Log subagent stop hook info to stderr (visible in hooktest output)
    eprintln!("SubagentStop hook triggered!");
    eprintln!("Session ID: {}", subagent_stop.session_id);
    eprintln!("Stop hook active: {}", subagent_stop.stop_hook_active);

    // Check if we're already in a stop hook to prevent infinite loops
    if subagent_stop.stop_hook_active {
        eprintln!("Already in subagent stop hook, allowing stop to prevent loop");
        subagent_stop.allow().respond();
    }

    // For demonstration, block subagent from stopping if session ID contains "continue"
    if subagent_stop.session_id.contains("continue") {
        eprintln!("Blocking subagent stop - session requires continuation");
        subagent_stop
            .block("Subagent task not yet complete, continuing...")
            .respond();
    }

    // Otherwise, allow subagent to stop normally
    eprintln!("Allowing subagent to stop normally");
    subagent_stop.allow().respond();
}
