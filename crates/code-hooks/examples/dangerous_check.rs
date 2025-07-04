use code_hooks::{HookResponse, Input, PreToolUse, PreToolUseOutput, Result};

fn main() -> Result<()> {
    // Read PreToolUse input from stdin
    let input = PreToolUse::read()?;

    // Check if it's a Bash command
    if input.tool_name == "Bash" {
        if let Some(command) = input.tool_input.get("command").and_then(|v| v.as_str()) {
            // Check for dangerous patterns
            if command.contains("rm -rf")
                || command.contains("dd if=")
                || command.contains(":(){ :|:& };:")
            {
                eprintln!("Dangerous command detected: {command}");
                PreToolUseOutput::block(
                    "This command appears to be dangerous and has been blocked for safety.",
                )
                .respond();
            }
        }
    }

    input.approve("Command validated and approved").respond();
}
