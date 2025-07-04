use crate::color::ColorMode;
use crate::execute::execute_hook;
use crate::output::Output;
use anyhow::Result;
use code_hooks::PreToolUse;
use std::collections::HashMap;

pub fn run_pretooluse_hook(
    session_id: String,
    transcript_path: String,
    tool_name: String,
    tool_input_str: String,
    hook_args: Vec<String>,
    color_mode: ColorMode,
) -> Result<()> {
    let mut out = Output::new(color_mode);

    // Parse the tool input JSON into a HashMap
    let tool_input: HashMap<String, serde_json::Value> = serde_json::from_str(&tool_input_str)?;

    // Create the hook input using the PreToolUse struct
    let hook_input = PreToolUse {
        session_id,
        transcript_path,
        tool_name,
        tool_input,
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
                "approve" => {
                    out.write("Decision: ")?;
                    out.success("APPROVE")?;
                    out.newline()?;

                    if let Some(reason) = hook_output.get("reason").and_then(|r| r.as_str()) {
                        out.label("User sees", reason)?;
                        out.dimmed("Claude sees: (nothing, tool proceeds)")?;
                    }
                }
                "block" => {
                    out.write("Decision: ")?;
                    out.error("BLOCK")?;
                    out.newline()?;

                    if let Some(reason) = hook_output.get("reason").and_then(|r| r.as_str()) {
                        out.label("User sees", "Tool blocked by hook")?;
                        out.label("Claude sees", reason)?;
                    }
                }
                _ => {
                    out.label("Decision", &format!("Unknown ({decision})"))?;
                }
            }
        } else {
            out.dimmed("Decision: NONE (follows normal permission flow)")?;
        }

        if hook_output.get("continue").and_then(|c| c.as_bool()) == Some(false) {
            out.newline()?;
            out.error("Claude would STOP processing")?;
            out.newline()?;
            if let Some(reason) = hook_output.get("stopReason").and_then(|r| r.as_str()) {
                out.label("Stop reason shown to user", reason)?;
            }
        }
    }

    Ok(())
}
