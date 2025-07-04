use crate::color::ColorMode;
use crate::execute::execute_hook;
use crate::output::Output;
use anyhow::Result;
use code_hooks::Notification;

pub fn run_notification_hook(
    session_id: String,
    transcript_path: String,
    message: String,
    title: String,
    hook_args: Vec<String>,
    color_mode: ColorMode,
) -> Result<()> {
    let mut out = Output::new(color_mode);

    // Create the hook input using the Notification struct
    let hook_input = Notification {
        session_id,
        transcript_path,
        message,
        hook_event_name: title,
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

        // Check continue field
        if hook_output.get("continue").and_then(|c| c.as_bool()) == Some(false) {
            out.error("Claude would STOP processing")?;
            out.newline()?;
            if let Some(reason) = hook_output.get("stopReason").and_then(|r| r.as_str()) {
                out.label("Stop reason shown to user", reason)?;
            }
        } else {
            out.dimmed("Claude continues normally")?;
        }

        if hook_output.get("suppressOutput").and_then(|s| s.as_bool()) == Some(true) {
            out.newline()?;
            out.dimmed("Output would be hidden in transcript mode")?;
        }
    }

    Ok(())
}
