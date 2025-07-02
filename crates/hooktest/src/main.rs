use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};

#[derive(Parser)]
#[command(
    name = "hooktest",
    about = "Test utility for Claude Code hooks",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test a PreToolUse hook
    #[command(name = "pretooluse")]
    PreToolUse {
        /// Session ID for the hook
        #[arg(long)]
        sessionid: String,

        /// Transcript path for the hook
        #[arg(long, default_value = "/tmp/transcript.json")]
        transcript: String,

        /// Tool name being called
        #[arg(long, default_value = "Bash")]
        tool: String,

        /// Tool input as JSON string
        #[arg(long, default_value = r#"{"command": "echo 'test'"}"#)]
        input: String,

        /// Hook command and arguments (everything after --)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        hook_args: Vec<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::PreToolUse {
            sessionid,
            transcript,
            tool,
            input,
            hook_args,
        } => run_pretooluse_hook(sessionid, transcript, tool, input, hook_args),
    }
}

fn run_pretooluse_hook(
    session_id: String,
    transcript_path: String,
    tool_name: String,
    tool_input_str: String,
    hook_args: Vec<String>,
) -> Result<()> {
    // Parse the tool input JSON
    let tool_input: serde_json::Value = serde_json::from_str(&tool_input_str)?;

    // Create the hook input JSON
    let hook_input = json!({
        "session_id": session_id,
        "transcript_path": transcript_path,
        "tool_name": tool_name,
        "tool_input": tool_input
    });

    // Serialize to JSON
    let input_json = serde_json::to_string(&hook_input)?;

    // Execute the hook
    if hook_args.is_empty() {
        anyhow::bail!("No hook command provided. Use -- followed by the hook command.");
    }

    let mut cmd = Command::new(&hook_args[0]);
    if hook_args.len() > 1 {
        cmd.args(&hook_args[1..]);
    }

    println!("=== Running Hook ===");
    println!("Command: {} {}", hook_args[0], hook_args[1..].join(" "));
    println!("\n=== Input JSON ===");
    println!("{}", serde_json::to_string_pretty(&hook_input)?);
    println!("\n=== Execution ===");

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    // Write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input_json.as_bytes())?;
        stdin.flush()?;
    }

    // Wait for the process to complete
    let output = child.wait_with_output()?;

    println!("Exit Code: {}", output.status.code().unwrap_or(-1));

    if !output.stdout.is_empty() {
        println!("\n=== STDOUT ===");
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }

    if !output.stderr.is_empty() {
        println!("\n=== STDERR ===");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    // Parse the output if successful
    if output.status.success() && !output.stdout.is_empty() {
        match serde_json::from_slice::<serde_json::Value>(&output.stdout) {
            Ok(hook_output) => {
                println!("\n=== Hook Output (Parsed) ===");
                println!("{}", serde_json::to_string_pretty(&hook_output)?);

                println!("\n=== What Claude/User Would See ===");

                // Parse decision field
                if let Some(decision) = hook_output.get("decision").and_then(|d| d.as_str()) {
                    match decision {
                        "approve" => {
                            println!("Decision: APPROVE");
                            if let Some(reason) = hook_output.get("reason").and_then(|r| r.as_str())
                            {
                                println!("User sees: {reason}");
                                println!("Claude sees: (nothing, tool proceeds)");
                            }
                        }
                        "block" => {
                            println!("Decision: BLOCK");
                            if let Some(reason) = hook_output.get("reason").and_then(|r| r.as_str())
                            {
                                println!("User sees: Tool blocked by hook");
                                println!("Claude sees: {reason}");
                            }
                        }
                        _ => {
                            println!("Decision: Unknown ({decision})");
                        }
                    }
                } else {
                    println!("Decision: NONE (follows normal permission flow)");
                }

                if hook_output.get("continue").and_then(|c| c.as_bool()) == Some(false) {
                    println!("\nClaude would STOP processing");
                    if let Some(reason) = hook_output.get("stopReason").and_then(|r| r.as_str()) {
                        println!("Stop reason shown to user: {reason}");
                    }
                }
            }
            Err(e) => {
                println!("\n=== Hook Output (Raw - Failed to parse) ===");
                println!("{}", String::from_utf8_lossy(&output.stdout));
                println!("Parse error: {e}");
            }
        }
    }

    Ok(())
}
