# Code Hooks

Rust toolkit for building hooks for [Claude Code](https://claude.ai/code).
Hooks are shell commands that execute at various points in Claude Code's
lifecycle.

## Crates

- **[code-hooks](./crates/code-hooks/)**: Core library for building Claude Code
  hooks
- **[claude-transcript](./crates/claude-transcript/)**: Parse and analyze
  Claude conversation transcripts  
- **[hooktest](./crates/hooktest/)**: Test hooks during development
- **[rust-hook](./crates/rust-hook/)**: Example hook that formats and lints
  Rust code

## Quick Start

```rust
use code_hooks::*;

fn main() -> Result<()> {
    let input = PreToolUse::read()?;
    
    if input.tool_name == "Bash" {
        if let Some(cmd) = input.tool_input.get("command").and_then(|v| v.as_str()) {
            if cmd.contains("rm -rf /") {
                return input.block("Dangerous command").respond();
            }
        }
    }
    
    input.approve("OK").respond()
}
```

Test with hooktest:
```bash
hooktest pretool --tool Bash --tool-input command="ls" -- ./my-hook
```

