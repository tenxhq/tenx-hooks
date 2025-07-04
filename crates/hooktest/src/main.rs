mod output;
mod posttool;
mod pretool;

use anyhow::Result;
use clap::{Parser, Subcommand};

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::PreTool {
            sessionid,
            transcript,
            tool,
            input,
            hook_args,
        } => pretool::run_pretooluse_hook(sessionid, transcript, tool, input, hook_args),
        Commands::PostTool {
            sessionid,
            transcript,
            tool,
            input,
            response,
            hook_args,
        } => {
            posttool::run_posttooluse_hook(sessionid, transcript, tool, input, response, hook_args)
        }
    }
}
