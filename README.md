# tenx-hooks

A Rust library for building hooks for [Claude Code](https://claude.ai/code),
Anthropic's official CLI for Claude.

## Overview

`tenx-hooks` provides a simple, type-safe way to write hooks that extend Claude
Code's functionality. Hooks are user-defined shell commands that execute at
various points in Claude Code's lifecycle, enabling you to:

- **Automate code formatting** - Run formatters after file edits
- **Enforce coding standards** - Validate changes before they're applied
- **Add custom notifications** - Get notified your way when Claude needs input
- **Implement security policies** - Block modifications to sensitive files
- **Track operations** - Log all commands and changes for compliance

## Features

- Type-safe Rust API for Claude Code hooks
- Automatic JSON parsing of hook inputs
- Structured output with decision control
- Built-in input validation and error handling
- Support for all hook events (PreToolUse, PostToolUse, Notification, Stop)
- Zero-cost abstractions with minimal dependencies

## Installation

Add `tenx-hooks` to your `Cargo.toml`:

```toml
[dependencies]
tenx-hooks = "0.0.1"
```

## Quick Start

Here's a simple hook that validates Bash commands before execution:

```rust
use tenx_hooks::{Hook, PreToolUseOutput, Decision, Result};

fn main() -> Result<()> {
    let hook = Hook::new();
    
    // Read and parse the PreToolUse input
    let input = hook.pre_tool_use()?;
    
    if input.tool_name == "Bash" {
        if let Some(command) = input.tool_input.get("command").and_then(|v| v.as_str()) {
            if command.contains("rm -rf") {
                // Block the operation with a reason
                hook.respond(PreToolUseOutput {
                    decision: Some(Decision::Block),
                    reason: Some("Dangerous command detected: rm -rf is not allowed".to_string()),
                    ..Default::default()
                })?;
                return Ok(());
            }
        }
    }
    
    // Approve the operation
    hook.respond(PreToolUseOutput {
        decision: Some(Decision::Approve),
        reason: Some("Command validated".to_string()),
        ..Default::default()
    })?;
    
    Ok(())
}
```

## Hook Events

`tenx-hooks` supports all Claude Code hook events with dedicated methods and output types:

### PreToolUse
Runs before tool execution. Can approve or block the operation.

```rust
use tenx_hooks::{Hook, PreToolUseOutput, Decision, Result};

fn main() -> Result<()> {
    let hook = Hook::new();
    let input = hook.pre_tool_use()?;
    
    // Block file writes to protected directories
    if input.tool_name == "Write" {
        if let Some(path) = input.tool_input.get("file_path").and_then(|v| v.as_str()) {
            if path.starts_with("/etc/") {
                hook.respond(PreToolUseOutput {
                    decision: Some(Decision::Block),
                    reason: Some("Cannot write to system directories".to_string()),
                    ..Default::default()
                })?;
                return Ok(());
            }
        }
    }
    
    // Approve by default
    hook.respond(PreToolUseOutput::approve("Allowed"))?;
    Ok(())
}
```

### PostToolUse
Runs after successful tool execution. Perfect for formatting or validation.

```rust
use tenx_hooks::{Hook, PostToolUseOutput, Decision, Result};

fn main() -> Result<()> {
    let hook = Hook::new();
    let input = hook.post_tool_use()?;
    
    // Run formatter after file edits
    if matches!(input.tool_name.as_str(), "Write" | "Edit" | "MultiEdit") {
        // Run your formatter here
        std::process::Command::new("cargo")
            .arg("fmt")
            .status()?;
        
        hook.respond(PostToolUseOutput {
            suppress_output: true,
            ..Default::default()
        })?;
    }
    
    Ok(())
}
```

### Notification
Customizes how you receive notifications from Claude Code.

```rust
use tenx_hooks::{Hook, NotificationOutput, Result};

fn main() -> Result<()> {
    let hook = Hook::new();
    let input = hook.notification()?;
    
    // Send to your preferred notification system
    send_desktop_notification(&input.title, &input.message)?;
    
    // Suppress output in transcript mode
    hook.respond(NotificationOutput {
        suppress_output: true,
        ..Default::default()
    })?;
    
    Ok(())
}
```

### Stop
Runs when Claude finishes responding. Can request continuation.

```rust
use tenx_hooks::{Hook, StopOutput, Decision, Result};

fn main() -> Result<()> {
    let hook = Hook::new();
    let input = hook.stop()?;
    
    // Continue if tests are failing
    if !input.stop_hook_active && tests_failing()? {
        hook.respond(StopOutput {
            decision: Some(Decision::Block),
            reason: Some("Tests are failing. Please fix them before stopping.".to_string()),
            ..Default::default()
        })?;
    }
    
    Ok(())
}
```

## Output Types

Each hook event has its own output type with specific fields:

### PreToolUseOutput
```rust
pub struct PreToolUseOutput {
    pub decision: Option<Decision>,  // Approve or Block
    pub reason: Option<String>,      // Explanation for decision
    pub continue_: Option<bool>,     // Whether Claude should continue
    pub stop_reason: Option<String>, // Message when continue is false
    pub suppress_output: Option<bool>, // Hide output from transcript
}

// Convenience constructors
PreToolUseOutput::approve("Allowed");
PreToolUseOutput::block("Not allowed");
```

### PostToolUseOutput
```rust
pub struct PostToolUseOutput {
    pub decision: Option<Decision>,  // Block only (tool already ran)
    pub reason: Option<String>,      // Feedback for Claude
    pub continue_: Option<bool>,     // Whether Claude should continue
    pub stop_reason: Option<String>, // Message when continue is false
    pub suppress_output: Option<bool>, // Hide output from transcript
}
```

### NotificationOutput
```rust
pub struct NotificationOutput {
    pub continue_: Option<bool>,     // Whether Claude should continue
    pub stop_reason: Option<String>, // Message when continue is false
    pub suppress_output: Option<bool>, // Hide output from transcript
}
```

### StopOutput
```rust
pub struct StopOutput {
    pub decision: Option<Decision>,  // Block to prevent stopping
    pub reason: Option<String>,      // Required when blocking
    pub continue_: Option<bool>,     // Whether Claude should continue
    pub stop_reason: Option<String>, // Message when continue is false
    pub suppress_output: Option<bool>, // Hide output from transcript
}
```

## Examples

### Command Logger
```rust
use tenx_hooks::{Hook, PreToolUseOutput, Result};
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> Result<()> {
    let hook = Hook::new();
    let input = hook.pre_tool_use()?;
    
    if input.tool_name == "Bash" {
        if let Some(cmd) = input.tool_input.get("command").and_then(|v| v.as_str()) {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open("command_history.log")?;
            writeln!(file, "{}: {}", chrono::Local::now(), cmd)?;
        }
    }
    
    // Log silently without showing in transcript
    hook.respond(PreToolUseOutput {
        suppress_output: true,
        ..Default::default()
    })?;
    
    Ok(())
}
```

### Code Style Enforcer
```rust
use tenx_hooks::{Hook, PreToolUseOutput, Decision, Result};

fn main() -> Result<()> {
    let hook = Hook::new();
    let input = hook.pre_tool_use()?;
    
    if matches!(input.tool_name.as_str(), "Write" | "Edit") {
        if let Some(content) = input.tool_input.get("content").and_then(|v| v.as_str()) {
            // Check for tabs in Python files
            if let Some(path) = input.tool_input.get("file_path").and_then(|v| v.as_str()) {
                if path.ends_with(".py") && content.contains('\t') {
                    hook.respond(PreToolUseOutput::block(
                        "Python files must use spaces, not tabs"
                    ))?;
                    return Ok(());
                }
            }
        }
    }
    
    hook.respond(PreToolUseOutput::approve("Style check passed"))?;
    Ok(())
}
```

## Related

- [Claude Code Documentation](https://docs.anthropic.com/en/docs/claude-code)
- [Claude Code Hooks Guide](https://docs.anthropic.com/en/docs/claude-code/hooks)
- [Model Context Protocol (MCP)](https://modelcontextprotocol.io)
