use tenx_hooks::{PreToolUse, output::PreToolUseOutput};

fn main() {
    let input = r#"{
        "session_id": "test-session",
        "transcript_path": "/tmp/transcript.json",
        "tool_name": "Bash",
        "tool_input": {
            "command": "echo 'Hello, World!'"
        }
    }"#;

    let pre_tool_use: PreToolUse = serde_json::from_str(input).unwrap();
    println!("Parsed tool: {}", pre_tool_use.tool_name);
    println!(
        "Tool input: {}",
        serde_json::to_string(&pre_tool_use.tool_input).unwrap()
    );

    let approval = PreToolUseOutput::approve("Command looks safe");
    println!("Result: {}", serde_json::to_string(&approval).unwrap());
}
