mod color;
mod execute;
mod input;
mod log;
mod notification;
mod output;
mod posttool;
mod pretool;
mod stop;
mod subagent_stop;
mod transcript;

use anyhow::Result;
use clap::{Parser, Subcommand};
use color::ColorMode;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Parser)]
#[command(
    name = "hooktest",
    about = "Test utility for Claude Code hooks",
    version
)]
struct Cli {
    /// Enable colored output
    #[arg(long, global = true, conflicts_with = "no_color")]
    color: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test a PreToolUse hook
    #[command(name = "pretool")]
    PreTool {
        /// Session ID for the hook (generated if not provided)
        #[arg(long)]
        sessionid: Option<String>,

        /// Transcript path for the hook
        #[arg(long, default_value = "/tmp/transcript.json")]
        transcript: String,

        /// Tool name being called
        #[arg(long, default_value = "Bash")]
        tool: String,

        /// Tool input as key=value pairs (e.g., --tool-input command="echo hello")
        #[arg(long = "tool-input", value_name = "KEY=VALUE")]
        tool_input: Vec<String>,

        /// Tool input as key=json pairs (e.g., --tool-input-json args='["one", "two"]')
        #[arg(long = "tool-input-json", value_name = "KEY=JSON")]
        tool_input_json: Vec<String>,

        /// Hook command and arguments (everything after --)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        hook_args: Vec<String>,
    },
    /// Test a PostToolUse hook
    #[command(name = "posttool")]
    PostTool {
        /// Session ID for the hook (generated if not provided)
        #[arg(long)]
        sessionid: Option<String>,

        /// Transcript path for the hook
        #[arg(long, default_value = "/tmp/transcript.json")]
        transcript: String,

        /// Tool name that was called
        #[arg(long, default_value = "Bash")]
        tool: String,

        /// Tool input as key=value pairs (e.g., --tool-input command="echo hello")
        #[arg(long = "tool-input", value_name = "KEY=VALUE")]
        tool_input: Vec<String>,

        /// Tool input as key=json pairs (e.g., --tool-input-json args='["one", "two"]')
        #[arg(long = "tool-input-json", value_name = "KEY=JSON")]
        tool_input_json: Vec<String>,

        /// Tool response as key=value pairs (e.g., --tool-response output="test")
        #[arg(long = "tool-response", value_name = "KEY=VALUE")]
        tool_response: Vec<String>,

        /// Tool response as key=json pairs (e.g., --tool-response-json exitCode=0)
        #[arg(long = "tool-response-json", value_name = "KEY=JSON")]
        tool_response_json: Vec<String>,

        /// Hook command and arguments (everything after --)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        hook_args: Vec<String>,
    },
    /// Test a Notification hook
    #[command(name = "notification")]
    Notification {
        /// Session ID for the hook (generated if not provided)
        #[arg(long)]
        sessionid: Option<String>,

        /// Transcript path for the hook
        #[arg(long, default_value = "/tmp/transcript.json")]
        transcript: String,

        /// Notification message
        #[arg(long, default_value = "Claude needs permission to run a command")]
        message: String,

        /// Notification title
        #[arg(long, default_value = "Claude Code")]
        title: String,

        /// Hook command and arguments (everything after --)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        hook_args: Vec<String>,
    },
    /// Test a Stop hook
    #[command(name = "stop")]
    Stop {
        /// Session ID for the hook (generated if not provided)
        #[arg(long)]
        sessionid: Option<String>,

        /// Transcript path for the hook
        #[arg(long, default_value = "/tmp/transcript.json")]
        transcript: String,

        /// Whether stop hook is already active (to prevent loops)
        #[arg(long, default_value = "false")]
        active: bool,

        /// Hook command and arguments (everything after --)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        hook_args: Vec<String>,
    },
    /// Test a SubagentStop hook
    #[command(name = "subagentstop")]
    SubagentStop {
        /// Session ID for the hook (generated if not provided)
        #[arg(long)]
        sessionid: Option<String>,

        /// Transcript path for the hook
        #[arg(long, default_value = "/tmp/transcript.json")]
        transcript: String,

        /// Whether stop hook is already active (to prevent loops)
        #[arg(long, default_value = "false")]
        active: bool,

        /// Hook command and arguments (everything after --)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        hook_args: Vec<String>,
    },
    /// Read an event from stdin and appendit to a JSONL log file
    #[command(name = "log")]
    Log {
        /// Event type to log (pretool, posttool, notification, stop, subagentstop)
        event: String,

        /// File path to write the log
        filepath: String,

        /// Optional path to read and rewrite transcript
        #[arg(long)]
        transcript: Option<String>,
    },
    /// Format and display transcript files
    #[command(name = "transcript")]
    Transcript {
        /// Paths to the transcript JSONL files
        paths: Vec<String>,

        /// Enable strict validation to check for missing fields
        #[arg(long)]
        strict: bool,
    },
}

/// Generate a session ID based on current timestamp
fn generate_session_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("test-session-{}", timestamp)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let color_mode = ColorMode::from_flags(cli.color, cli.no_color);

    match cli.command {
        Commands::PreTool {
            sessionid,
            transcript,
            tool,
            tool_input,
            tool_input_json,
            hook_args,
        } => {
            let session_id = sessionid.unwrap_or_else(generate_session_id);

            // Handle tool input
            let tool_input_map = if tool_input.is_empty() && tool_input_json.is_empty() {
                // No inputs provided, use default based on tool
                let mut default_map = std::collections::HashMap::new();
                if tool == "Bash" {
                    default_map.insert(
                        "command".to_string(),
                        serde_json::Value::String("echo 'test'".to_string()),
                    );
                }
                default_map
            } else {
                // Combine tool-input and tool-input-json
                input::combine_inputs(None, &tool_input, &tool_input_json)?
            };

            pretool::run_pretooluse_hook(
                session_id,
                transcript,
                tool,
                tool_input_map,
                hook_args,
                color_mode,
            )
        }
        Commands::PostTool {
            sessionid,
            transcript,
            tool,
            tool_input,
            tool_input_json,
            tool_response,
            tool_response_json,
            hook_args,
        } => {
            let session_id = sessionid.unwrap_or_else(generate_session_id);

            // Handle tool input
            let tool_input_map = if tool_input.is_empty() && tool_input_json.is_empty() {
                // No inputs provided, use default based on tool
                let mut default_map = std::collections::HashMap::new();
                if tool == "Bash" {
                    default_map.insert(
                        "command".to_string(),
                        serde_json::Value::String("echo 'test'".to_string()),
                    );
                }
                default_map
            } else {
                // Combine tool-input and tool-input-json
                input::combine_inputs(None, &tool_input, &tool_input_json)?
            };

            // Handle tool response
            let tool_response_map = if tool_response.is_empty() && tool_response_json.is_empty() {
                // No response provided, use default
                let mut default_map = std::collections::HashMap::new();
                default_map.insert(
                    "output".to_string(),
                    serde_json::Value::String("test\n".to_string()),
                );
                default_map
            } else {
                // Combine tool-response and tool-response-json
                input::combine_inputs(None, &tool_response, &tool_response_json)?
            };

            posttool::run_posttooluse_hook(
                session_id,
                transcript,
                tool,
                tool_input_map,
                tool_response_map,
                hook_args,
                color_mode,
            )
        }
        Commands::Notification {
            sessionid,
            transcript,
            message,
            title,
            hook_args,
        } => {
            let session_id = sessionid.unwrap_or_else(generate_session_id);
            notification::run_notification_hook(
                session_id, transcript, message, title, hook_args, color_mode,
            )
        }
        Commands::Stop {
            sessionid,
            transcript,
            active,
            hook_args,
        } => {
            let session_id = sessionid.unwrap_or_else(generate_session_id);
            stop::run_stop_hook(session_id, transcript, active, hook_args, color_mode)
        }
        Commands::SubagentStop {
            sessionid,
            transcript,
            active,
            hook_args,
        } => {
            let session_id = sessionid.unwrap_or_else(generate_session_id);
            subagent_stop::run_subagent_stop_hook(
                session_id, transcript, active, hook_args, color_mode,
            )
        }
        Commands::Log {
            event,
            filepath,
            transcript,
        } => log::run_log_hook(event, filepath, transcript, color_mode),
        Commands::Transcript { paths, strict } => {
            transcript::display_transcripts(paths, color_mode, strict)
        }
    }
}
