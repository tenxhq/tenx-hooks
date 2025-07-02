use tenx_hooks::{Hook, Result, output::PreToolUseOutput};

fn main() -> Result<()> {
    let hook = Hook::new();

    // Read PreToolUse input from stdin
    let input = hook.pre_tooluse()?;

    // Check if it's a Bash command
    if input.tool_name == "Bash" {
        if let Some(command) = input.tool_input.get("command").and_then(|v| v.as_str()) {
            // Check for dangerous patterns
            if command.contains("rm -rf")
                || command.contains("dd if=")
                || command.contains(":(){ :|:& };:")
            {
                eprintln!("Dangerous command detected: {command}");
                let response = PreToolUseOutput::block(
                    "This command appears to be dangerous and has been blocked for safety.",
                );
                hook.respond(response)?;
                return Ok(());
            }
        }
    }

    // Otherwise approve
    let approval = PreToolUseOutput::approve("Command validated and approved");
    hook.respond(approval)?;

    Ok(())
}
