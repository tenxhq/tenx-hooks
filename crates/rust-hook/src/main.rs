use anyhow::Result;
use clap::{Parser, Subcommand};
use code_hooks::{HookResponse, Input, PostToolUse, PostToolUseOutput, Stop, TranscriptReader};
use rust_hook::is_rust_file;
use std::process::Command;

#[derive(Parser)]
#[command(name = "rust-hook")]
#[command(about = "A Claude Code hook that formats and lints Rust code")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Handle post-tool-use events
    Posttool,
    /// Handle stop events
    Stop,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Posttool => handle_posttool(),
        Commands::Stop => handle_stop(),
    }
}

fn handle_posttool() -> Result<()> {
    eprintln!("[rust-hook] Starting posttool handler");
    let input = PostToolUse::read()?;
    eprintln!("[rust-hook] Tool: {}", input.tool_name);

    // Only process Edit and MultiEdit tools
    if input.tool_name != "Edit" && input.tool_name != "MultiEdit" {
        eprintln!("[rust-hook] Not an Edit/MultiEdit tool, passing through");
        PostToolUseOutput::passthrough().respond();
    }

    // Extract file path from tool response
    let file_path = input
        .tool_response
        .get("filePath")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    eprintln!("[rust-hook] File path: {file_path}");

    // Check if the file is a Rust file
    if !is_rust_file(file_path) {
        eprintln!("[rust-hook] Not a Rust file, passing through");
        PostToolUseOutput::passthrough().respond();
    }

    eprintln!("[rust-hook] Processing Rust file: {file_path}");

    // Run formatting and linting on the entire project
    let feedback_messages = run_rust_tools()?;

    if !feedback_messages.is_empty() {
        eprintln!(
            "[rust-hook] Found {} issues, blocking",
            feedback_messages.len()
        );
        PostToolUseOutput::block(&feedback_messages.join("\n\n")).respond()
    } else {
        eprintln!("[rust-hook] No issues found, passing through");
        PostToolUseOutput::passthrough().respond()
    }
}

fn handle_stop() -> Result<()> {
    eprintln!("[rust-hook] Starting stop handler");
    let input = Stop::read()?;

    // Check if any Rust files were edited in this session
    if !has_edited_rust_files(&input)? {
        eprintln!("[rust-hook] No Rust files edited, allowing stop");
        input.allow().respond();
    }

    eprintln!("[rust-hook] Rust files were edited, checking project");

    // Run formatting and linting on the entire project
    let feedback_messages = run_rust_tools()?;

    if !feedback_messages.is_empty() {
        eprintln!(
            "[rust-hook] Found {} issues, blocking stop",
            feedback_messages.len()
        );
        let message = format!(
            "Rust formatting/linting issues found:\n\n{}\n\nPlease fix these issues.",
            feedback_messages.join("\n\n")
        );
        input.block(&message).respond()
    } else {
        eprintln!("[rust-hook] No issues found, allowing stop");
        input.allow().respond()
    }
}

fn has_edited_rust_files(input: &Stop) -> Result<bool> {
    use claude_transcript::{TranscriptEntry, TranscriptMessage};

    let transcript = input.read_transcript()?;

    for entry in transcript {
        if let TranscriptEntry::Assistant(assistant_entry) = entry {
            if let TranscriptMessage::Assistant {
                tool_uses: Some(tool_uses),
                ..
            } = assistant_entry.message
            {
                for tool_use in tool_uses {
                    if tool_use.tool_name == "Edit" || tool_use.tool_name == "MultiEdit" {
                        if let Some(file_path) = tool_use
                            .tool_input
                            .get("file_path")
                            .and_then(|v| v.as_str())
                        {
                            if is_rust_file(file_path) {
                                eprintln!("[rust-hook] Found edited Rust file: {file_path}");
                                return Ok(true);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(false)
}

fn run_rust_tools() -> Result<Vec<String>> {
    let mut feedback_messages = Vec::new();

    // Run cargo fmt --all
    eprintln!("[rust-hook] Running cargo fmt...");
    match run_cargo_fmt() {
        Ok(output) => {
            if !output.success {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("[rust-hook] cargo fmt failed with output:\n{stderr}");
                feedback_messages.push(format!("cargo fmt failed:\n{stderr}"));
            } else {
                eprintln!("[rust-hook] cargo fmt succeeded");
            }
        }
        Err(e) => {
            eprintln!("[rust-hook] Error running cargo fmt: {e}");
            feedback_messages.push(format!("Failed to run cargo fmt: {e}"));
        }
    }

    // Run cargo clippy
    eprintln!("[rust-hook] Running cargo clippy...");
    match run_cargo_clippy() {
        Ok(output) => {
            if !output.success {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("[rust-hook] cargo clippy found issues:\n{stderr}");
                feedback_messages.push(format!("cargo clippy found warnings:\n{stderr}"));
            } else {
                eprintln!("[rust-hook] cargo clippy succeeded");
            }
        }
        Err(e) => {
            eprintln!("[rust-hook] Error running cargo clippy: {e}");
            feedback_messages.push(format!("Failed to run cargo clippy: {e}"));
        }
    }

    eprintln!(
        "[rust-hook] Total feedback messages: {}",
        feedback_messages.len()
    );
    Ok(feedback_messages)
}

// Simple struct to hold command output
struct CommandOutput {
    success: bool,
    stderr: Vec<u8>,
}

fn run_cargo_fmt() -> Result<CommandOutput> {
    let mut cmd = Command::new("cargo");
    cmd.args(["fmt", "--all"]);

    log_command(&cmd, ".", "cargo fmt --all");

    let output = cmd.output()?;

    log_command_result(&output, "cargo fmt");

    Ok(CommandOutput {
        success: output.status.success(),
        stderr: output.stderr,
    })
}

fn run_cargo_clippy() -> Result<CommandOutput> {
    let mut cmd = Command::new("cargo");
    cmd.args(["clippy", "--tests", "--examples", "--fix", "--allow-dirty"]);

    log_command(
        &cmd,
        ".",
        "cargo clippy --tests --examples --fix --allow-dirty",
    );

    let output = cmd.output()?;

    log_command_result(&output, "cargo clippy");

    // Check if there are any warnings in stderr, even if clippy exited successfully
    let stderr_str = String::from_utf8_lossy(&output.stderr);
    let has_warnings = stderr_str.contains("warning:") || stderr_str.contains("error:");

    Ok(CommandOutput {
        success: !has_warnings,
        stderr: output.stderr,
    })
}

// Helper function to log command details
fn log_command(cmd: &Command, working_dir: &str, shell_cmd: &str) {
    eprintln!("[rust-hook] Full command: {cmd:?}");
    eprintln!("[rust-hook] Working directory: {working_dir}");
    eprintln!("[rust-hook] Equivalent shell command: {shell_cmd}");
}

// Helper function to log command output
fn log_command_result(output: &std::process::Output, cmd_name: &str) {
    let exit_code = output.status.code().unwrap_or(-1);
    eprintln!("[rust-hook] {cmd_name} exit code: {exit_code}");

    if !output.stdout.is_empty() {
        eprintln!(
            "[rust-hook] {cmd_name} stdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }

    if !output.stderr.is_empty() {
        eprintln!(
            "[rust-hook] {cmd_name} stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
