mod color;
mod execute;
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
    /// Test a PostToolUse hook
    #[command(name = "posttool")]
    PostTool {
        /// Session ID for the hook
        #[arg(long)]
        sessionid: String,

        /// Transcript path for the hook
        #[arg(long, default_value = "/tmp/transcript.json")]
        transcript: String,

        /// Tool name that was called
        #[arg(long, default_value = "Bash")]
        tool: String,

        /// Tool input as JSON string
        #[arg(long, default_value = r#"{"command": "echo 'test'"}"#)]
        input: String,

        /// Tool response as JSON string
        #[arg(long, default_value = r#"{"output": "test\n"}"#)]
        response: String,

        /// Hook command and arguments (everything after --)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        hook_args: Vec<String>,
    },
    /// Test a Notification hook
    #[command(name = "notification")]
    Notification {
        /// Session ID for the hook
        #[arg(long)]
        sessionid: String,

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
        /// Session ID for the hook
        #[arg(long)]
        sessionid: String,

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
        /// Session ID for the hook
        #[arg(long)]
        sessionid: String,

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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let color_mode = ColorMode::from_flags(cli.color, cli.no_color);

    match cli.command {
        Commands::PreTool {
            sessionid,
            transcript,
            tool,
            input,
            hook_args,
        } => {
            pretool::run_pretooluse_hook(sessionid, transcript, tool, input, hook_args, color_mode)
        }
        Commands::PostTool {
            sessionid,
            transcript,
            tool,
            input,
            response,
            hook_args,
        } => posttool::run_posttooluse_hook(
            sessionid, transcript, tool, input, response, hook_args, color_mode,
        ),
        Commands::Notification {
            sessionid,
            transcript,
            message,
            title,
            hook_args,
        } => notification::run_notification_hook(
            sessionid, transcript, message, title, hook_args, color_mode,
        ),
        Commands::Stop {
            sessionid,
            transcript,
            active,
            hook_args,
        } => stop::run_stop_hook(sessionid, transcript, active, hook_args, color_mode),
        Commands::SubagentStop {
            sessionid,
            transcript,
            active,
            hook_args,
        } => subagent_stop::run_subagent_stop_hook(
            sessionid, transcript, active, hook_args, color_mode,
        ),
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
