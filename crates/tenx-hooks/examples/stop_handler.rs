use tenx_hooks::{HookResponse, Input, Result, Stop};

fn main() -> Result<()> {
    // Read the hook input from stdin
    let stop = Stop::read()?;

    // Log stop hook info to stderr (visible in hooktest output)
    eprintln!("Stop hook triggered!");
    eprintln!("Session ID: {}", stop.session_id);
    eprintln!("Stop hook active: {}", stop.stop_hook_active);

    // Check if we're already in a stop hook to prevent infinite loops
    if stop.stop_hook_active {
        eprintln!("Already in stop hook, allowing stop to prevent loop");
        stop.allow().respond();
    }

    // For demonstration, block Claude from stopping if session ID contains "continue"
    if stop.session_id.contains("continue") {
        eprintln!("Blocking stop - session requires continuation");
        stop.block("Task not yet complete, continuing...").respond();
    }

    // Otherwise, allow Claude to stop normally
    eprintln!("Allowing Claude to stop normally");
    stop.allow().respond();
}