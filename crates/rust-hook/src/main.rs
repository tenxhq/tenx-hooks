use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use code_hooks::{HookResponse, Input, PostToolUse, PostToolUseOutput, Stop, TranscriptReader};
use std::path::Path;
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
    let input = PostToolUse::read()?;

    // Only process Edit and MultiEdit tools
    if input.tool_name != "Edit" && input.tool_name != "MultiEdit" {
        PostToolUseOutput::passthrough().respond();
    }

    // Extract file path from tool response
    let file_path = input
        .tool_response
        .get("filePath")
        .and_then(|v| v.as_str())
        .context("Failed to get file path from tool response")?;

    // Check if the file is a Rust file
    if !file_path.ends_with(".rs") {
        PostToolUseOutput::passthrough().respond();
    }

    // Check if the file exists
    if !Path::new(file_path).exists() {
        PostToolUseOutput::passthrough().respond();
    }

    // Run formatting and linting
    let feedback_messages = run_rust_tools(file_path)?;

    // If there are any feedback messages, block and provide feedback
    if !feedback_messages.is_empty() {
        PostToolUseOutput::block(&feedback_messages.join("\n\n")).respond();
    }

    // Otherwise, pass through
    PostToolUseOutput::passthrough().respond();
}

fn handle_stop() -> Result<()> {
    let input = Stop::read()?;

    // Get all Rust files that were edited in this session
    let rust_files = get_edited_rust_files(&input)?;

    if rust_files.is_empty() {
        input.allow().respond();
    }

    let mut all_feedback = Vec::new();

    // Run formatting and linting on each file
    for file_path in &rust_files {
        if Path::new(file_path).exists() {
            let feedback_messages = run_rust_tools(file_path)?;
            if !feedback_messages.is_empty() {
                all_feedback.push(format!(
                    "Issues in {}:\n{}",
                    file_path,
                    feedback_messages.join("\n")
                ));
            }
        }
    }

    // If there are any issues, ask Claude to continue and fix them
    if !all_feedback.is_empty() {
        let message = format!(
            "Rust formatting/linting issues found:\n\n{}\n\nPlease fix these issues.",
            all_feedback.join("\n\n")
        );
        input.block(&message).respond();
    }

    input.allow().respond();
}

fn get_edited_rust_files(input: &Stop) -> Result<Vec<String>> {
    use code_hooks::transcript::{TranscriptEntry, TranscriptMessage};

    let mut rust_files = Vec::new();

    // Read the transcript to find all edited Rust files
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
                            if file_path.ends_with(".rs")
                                && !rust_files.contains(&file_path.to_string())
                            {
                                rust_files.push(file_path.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(rust_files)
}

fn run_rust_tools(file_path: &str) -> Result<Vec<String>> {
    let project_root = find_project_root(file_path);
    let mut feedback_messages = Vec::new();

    // Run cargo fmt
    match run_cargo_fmt(&project_root, file_path) {
        Ok(output) => {
            if !output.success {
                feedback_messages.push(format!(
                    "cargo fmt failed:\n{}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
        Err(e) => {
            feedback_messages.push(format!("Failed to run cargo fmt: {e}"));
        }
    }

    // Run cargo clippy --fix --allow-dirty
    match run_cargo_clippy(&project_root) {
        Ok(output) => {
            if !output.success {
                let stderr = String::from_utf8_lossy(&output.stderr);
                // Check if there are warnings (clippy returns non-zero exit code for warnings)
                if stderr.contains("warning") {
                    feedback_messages.push(format!("cargo clippy found warnings:\n{stderr}"));
                } else {
                    feedback_messages.push(format!("cargo clippy failed:\n{stderr}"));
                }
            }
        }
        Err(e) => {
            feedback_messages.push(format!("Failed to run cargo clippy: {e}"));
        }
    }

    Ok(feedback_messages)
}

fn find_project_root(file_path: &str) -> String {
    let path = Path::new(file_path);
    let mut current = path.parent();

    while let Some(dir) = current {
        if dir.join("Cargo.toml").exists() {
            return dir.to_string_lossy().to_string();
        }
        current = dir.parent();
    }

    // If no Cargo.toml found, use the directory containing the file
    path.parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string())
}

struct CommandOutput {
    success: bool,
    stderr: Vec<u8>,
}

fn run_cargo_fmt(project_root: &str, file_path: &str) -> Result<CommandOutput> {
    let output = Command::new("cargo")
        .arg("fmt")
        .arg("--")
        .arg(file_path)
        .current_dir(project_root)
        .output()
        .context("Failed to execute cargo fmt")?;

    Ok(CommandOutput {
        success: output.status.success(),
        stderr: output.stderr,
    })
}

fn run_cargo_clippy(project_root: &str) -> Result<CommandOutput> {
    let output = Command::new("cargo")
        .args(["clippy", "--fix", "--allow-dirty", "--", "-D", "warnings"])
        .current_dir(project_root)
        .output()
        .context("Failed to execute cargo clippy")?;

    Ok(CommandOutput {
        success: output.status.success(),
        stderr: output.stderr,
    })
}
