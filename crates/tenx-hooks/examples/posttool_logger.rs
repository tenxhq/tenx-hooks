use tenx_hooks::{HookResponse, Input, PostToolUse, PostToolUseOutput, Result};

fn main() -> Result<()> {
    // Read the hook input from stdin
    let hook = PostToolUse::read()?;

    // Log tool execution info to stderr (visible in hooktest output)
    eprintln!("Tool executed: {}", hook.tool_name);
    eprintln!("Session ID: {}", hook.session_id);

    // Log the tool response
    if let Some(output) = hook.tool_response.get("output") {
        eprintln!("Tool output: {output:?}");
    }

    // Check if command contains sensitive patterns
    if let Some(command) = hook.tool_input.get("command").and_then(|v| v.as_str()) {
        if command.contains("secret") || command.contains("password") {
            // Block the output from being shown to Claude
            let response = PostToolUseOutput::block(
                "Tool output contains potentially sensitive information. Review required.",
            );

            // Write the response
            response.respond();
        }
    }

    // Otherwise, passthrough the tool output
    PostToolUseOutput::passthrough().respond();
}
