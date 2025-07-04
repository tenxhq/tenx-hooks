use crate::execute::execute_hook;
use crate::output::Output;
use anyhow::Result;
use tenx_hooks::Stop;

pub fn run_stop_hook(
    session_id: String,
    transcript_path: String,
    stop_hook_active: bool,
    hook_args: Vec<String>,
) -> Result<()> {
    let mut out = Output::new();

    // Create the hook input using the Stop struct
    let hook_input = Stop {
        session_id,
        transcript_path,
        stop_hook_active,
    };

    // Serialize to JSON
    let input_json = serde_json::to_string(&hook_input)?;

    // Execute the hook and parse output
    if let Some(hook_output) = execute_hook(
        &mut out,
        &hook_args,
        &input_json,
        &serde_json::to_value(&hook_input)?,
    )? {
        out.h1("What Claude/User Would See")?;

        // Parse decision field
        if let Some(decision) = hook_output.get("decision").and_then(|d| d.as_str()) {
            match decision {
                "block" => {
                    out.write("Decision: ")?;
                    out.error("BLOCK")?;
                    out.newline()?;

                    if let Some(reason) = hook_output.get("reason").and_then(|r| r.as_str()) {
                        out.label("Claude continues with", reason)?;
                    }
                }
                _ => {
                    out.label("Decision", &format!("Unknown ({decision})"))?;
                }
            }
        } else {
            out.dimmed("Decision: NONE (Claude stops normally)")?;
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

    Ok(())
}
