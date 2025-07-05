# code-hooks

Core library for building Claude Code hooks in Rust. Provides type-safe interfaces for all hook types.


## Example

```rust
use code_hooks::{PreToolUse, HookResponse, Input, Result};

fn main() -> Result<()> {
    let input = PreToolUse::read()?;
    
    // Log to stderr (won't interfere with JSON output)
    eprintln!("Tool: {}, Session: {}", input.tool_name, input.session_id);
    
    // Check for dangerous commands
    if input.tool_name == "Bash" {
        if let Some(cmd) = input.tool_input.get("command").and_then(|v| v.as_str()) {
            if cmd.contains("rm -rf") {
                return input.block("Dangerous command blocked").respond();
            }
        }
    }
    
    // Approve the tool use
    input.approve("Command approved").respond()
}
```

## Hook Types

- `PreToolUse`: Before tool execution (can approve/block/modify)
- `PostToolUse`: After tool execution (process results)
- `Notification`: System notifications
- `Stop`: Claude Code stopping
- `SubagentStop`: Subagent stopping

