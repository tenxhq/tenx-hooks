use crate::output::Output;
use anyhow::Result;
use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn run_posttooluse_hook(
    session_id: String,
    transcript_path: String,
    tool_name: String,
    tool_input_str: String,
    tool_response_str: String,
    hook_args: Vec<String>,
) -> Result<()> {
    let mut out = Output::new();

    // Parse the tool input and response JSON
    let tool_input: serde_json::Value = serde_json::from_str(&tool_input_str)?;
    let tool_response: serde_json::Value = serde_json::from_str(&tool_response_str)?;

    // Create the hook input JSON
    let hook_input = json!({
        "session_id": session_id,
        "transcript_path": transcript_path,
        "tool_name": tool_name,
        "tool_input": tool_input,
        "tool_response": tool_response
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

    out.h1("Running Hook")?;
    out.label(
        "Command",
        &format!("{} {}", hook_args[0], hook_args[1..].join(" ")),
    )?;

    out.h1("Input JSON")?;
    out.json(&hook_input)?;

    out.h1("Execution")?;

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

    let exit_code = output.status.code().unwrap_or(-1);
    if output.status.success() {
        out.label("Exit Code", &format!("{exit_code} "))?;
        out.success("✓")?;
        out.newline()?;
    } else {
        out.label("Exit Code", &format!("{exit_code} "))?;
        out.error("✗")?;
        out.newline()?;
    }

    if !output.stdout.is_empty() {
        out.h1("STDOUT")?;
        out.block(String::from_utf8_lossy(&output.stdout).trim_end())?;
    }

    if !output.stderr.is_empty() {
        out.h1("STDERR")?;
        out.dimmed(String::from_utf8_lossy(&output.stderr).trim_end())?;
    }

    // Parse the output if successful
    if output.status.success() && !output.stdout.is_empty() {
        match serde_json::from_slice::<serde_json::Value>(&output.stdout) {
            Ok(hook_output) => {
                out.h1("Hook Output (Parsed)")?;
                out.json(&hook_output)?;

                out.h1("What Claude/User Would See")?;

                // Parse decision field
                if let Some(decision) = hook_output.get("decision").and_then(|d| d.as_str()) {
                    match decision {
                        "block" => {
                            out.write("Decision: ")?;
                            out.error("BLOCK")?;
                            out.newline()?;

                            if let Some(reason) = hook_output.get("reason").and_then(|r| r.as_str())
                            {
                                out.label(
                                    "User sees",
                                    "Tool succeeded, but hook provided feedback",
                                )?;
                                out.label("Claude sees", reason)?;
                            }
                        }
                        _ => {
                            out.label("Decision", &format!("Unknown ({decision})"))?;
                        }
                    }
                } else {
                    out.dimmed("Decision: NONE (tool output passed through)")?;
                }

                if hook_output.get("continue").and_then(|c| c.as_bool()) == Some(false) {
                    out.newline()?;
                    out.error("Claude would STOP processing")?;
                    out.newline()?;
                    if let Some(reason) = hook_output.get("stopReason").and_then(|r| r.as_str()) {
                        out.label("Stop reason shown to user", reason)?;
                    }
                }

                if hook_output.get("suppressOutput").and_then(|s| s.as_bool()) == Some(true) {
                    out.newline()?;
                    out.dimmed("Output would be hidden in transcript mode")?;
                }
            }
            Err(e) => {
                out.h1("Hook Output (Raw - Failed to parse)")?;
                out.block(String::from_utf8_lossy(&output.stdout).trim_end())?;
                out.error(&format!("Parse error: {e}"))?;
                out.newline()?;
            }
        }
    }

    Ok(())
}
