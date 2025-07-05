# code-hooks

A Rust library for building hooks for [Claude Code](https://claude.ai/code),
Anthropic's official CLI for Claude.

## Overview

`code-hooks` provides a type-safe way to write hooks that extend Claude Code's
functionality. Hooks are shell commands that execute at various points in
Claude Code's lifecycle, enabling you to automate formatting, enforce policies,
customize notifications, and more.


## Hook Events

- **PreToolUse**: Runs before tool execution. Can approve or block operations.
- **PostToolUse**: Runs after tool execution. Perfect for formatting or
  validation.
- **Notification**: Customizes how you receive notifications.
- **Stop**: Runs when Claude finishes responding. Can request continuation.
- **SubagentStop**: Runs when a subagent finishes responding. Same semantics as
  Stop but only for subagents.


## Example

```rust
use code_hooks::{HookResponse, Input, PreToolUse, Result};

fn main() -> Result<()> {
    let input = PreToolUse::read()?;

    // Log some info to stderr for debugging (won't interfere with JSON output)
    eprintln!("Hook received tool: {}", input.tool_name);
    eprintln!("Session ID: {}", input.session_id);

    input.approve("Command looks safe").respond();
}
```



## Related

- [Claude Code Hooks Guide](https://docs.anthropic.com/en/docs/claude-code/hooks)
