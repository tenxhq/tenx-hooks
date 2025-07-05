use crate::output::Output;
use anyhow::Result;
use std::io::Write;
use std::process::{Command, Stdio};

/// Spawn a hook process, feed it the given JSON input, and print execution details.
///
/// Returns the parsed JSON output if the process succeeded and produced valid JSON.
pub fn execute_hook(
    out: &mut Output,
    hook_args: &[String],
    input_json: &str,
    hook_input_value: &serde_json::Value,
) -> Result<Option<serde_json::Value>> {
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
    out.json(hook_input_value)?;

    out.h1("Execution")?;

    let mut child = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input_json.as_bytes())?;
        stdin.flush()?;
    }

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
        out.block(String::from_utf8_lossy(&output.stderr).trim_end())?;
    }

    if output.status.success() && !output.stdout.is_empty() {
        match serde_json::from_slice::<serde_json::Value>(&output.stdout) {
            Ok(json) => {
                out.h1("Hook Output (Parsed)")?;
                out.json(&json)?;
                return Ok(Some(json));
            }
            Err(e) => {
                out.h1("Hook Output (Raw - Failed to parse)")?;
                out.block(String::from_utf8_lossy(&output.stdout).trim_end())?;
                out.error(&format!("Parse error: {e}"))?;
                out.newline()?;
            }
        }
    }

    Ok(None)
}
