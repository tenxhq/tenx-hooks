use anyhow::Result;
use clap::Parser;
use claude_transcript::TranscriptEntry;
use claude_transcript::parse::parse_transcript_with_context;
use std::fs;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

#[derive(Clone, Copy)]
pub enum ColorMode {
    Always,
    Never,
    Auto,
}

impl ColorMode {
    pub fn from_flags(color: bool, no_color: bool) -> Self {
        if color {
            ColorMode::Always
        } else if no_color {
            ColorMode::Never
        } else {
            ColorMode::Auto
        }
    }

    pub fn should_colorize(&self) -> bool {
        match self {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => atty::is(atty::Stream::Stdout),
        }
    }
}

pub struct JsonHighlighter {
    ps: SyntaxSet,
    ts: ThemeSet,
    enabled: bool,
}

impl JsonHighlighter {
    pub fn new(color_mode: ColorMode) -> Self {
        Self {
            ps: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
            enabled: color_mode.should_colorize(),
        }
    }

    pub fn print_json(&self, json: &str) -> Result<()> {
        if self.enabled {
            let syntax = self.ps.find_syntax_by_extension("json").unwrap();
            let mut h = HighlightLines::new(syntax, &self.ts.themes["base16-ocean.dark"]);

            for line in json.lines() {
                let ranges: Vec<(Style, &str)> = h.highlight_line(line, &self.ps)?;
                let escaped = as_24_bit_terminal_escaped(&ranges[..], false);
                println!("{escaped}");
            }
        } else {
            print!("{json}");
        }
        Ok(())
    }
}

#[derive(Parser)]
#[command(name = "ttest", about = "Format and display transcript files", version)]
struct Cli {
    /// Enable colored output
    #[arg(long, global = true, conflicts_with = "no_color")]
    color: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Enable verification mode (only outputs validation errors)
    #[arg(long)]
    verify: bool,

    /// Paths to the transcript JSONL files
    paths: Vec<String>,
}

pub fn display_transcripts(paths: Vec<String>, color_mode: ColorMode, verify: bool) -> Result<()> {
    if paths.is_empty() {
        anyhow::bail!("No transcript files specified");
    }

    let multiple_files = paths.len() > 1;
    let mut had_errors = false;

    for (file_idx, path) in paths.iter().enumerate() {
        if multiple_files && !verify {
            // Print file header
            if file_idx > 0 {
                println!(); // Blank line between files
            }
            println!("\x1b[1;36m=== {path} ===\x1b[0m");
        }

        match display_single_transcript(path.clone(), color_mode, verify) {
            Ok(()) => {}
            Err(e) => {
                if verify {
                    eprintln!("{path}: {e}");
                } else {
                    eprintln!("\x1b[91mError processing {path}: {e}\x1b[0m");
                }
                had_errors = true;
                if verify {
                    std::process::exit(1);
                }
            }
        }
    }

    if had_errors && !verify {
        std::process::exit(1);
    }

    Ok(())
}

fn display_single_transcript(path: String, color_mode: ColorMode, verify: bool) -> Result<()> {
    let content = fs::read_to_string(&path)?;
    let highlighter = JsonHighlighter::new(color_mode);

    if verify {
        // Use the context parsing for detailed error information
        let parse_result = parse_transcript_with_context(&content);

        // If there are parsing errors, show those
        if !parse_result.errors.is_empty() {
            for error in &parse_result.errors {
                eprintln!("{}:{}: {}", path, error.line_number, error.json_error);
            }

            // Exit with error code if there were parsing errors
            std::process::exit(1);
        }
        // In verify mode, output nothing if validation passes
    } else {
        // Non-strict mode: parse and display what we can
        let parse_result = parse_transcript_with_context(&content);

        if !parse_result.errors.is_empty() {
            eprintln!(
                "\x1b[93mWarning: {} lines could not be parsed\x1b[0m",
                parse_result.errors.len()
            );
        }

        for (line_idx, line) in content.lines().enumerate() {
            if line.is_empty() {
                continue;
            }

            // Try to parse and pretty-print each line
            match serde_json::from_str::<serde_json::Value>(line) {
                Ok(value) => {
                    // Add line number
                    println!("\x1b[2m# Line {}\x1b[0m", line_idx + 1);

                    // If we can parse it as a transcript entry, show entry type
                    if let Ok(entry) = serde_json::from_value::<TranscriptEntry>(value.clone()) {
                        let entry_type = match entry {
                            TranscriptEntry::System(_) => "System entry",
                            TranscriptEntry::User(_) => "User entry",
                            TranscriptEntry::Assistant(_) => "Assistant entry",
                            TranscriptEntry::Summary(_) => "Summary entry",
                        };
                        println!("\x1b[94m{entry_type}\x1b[0m");
                    }

                    // Pretty-print the JSON
                    let pretty = serde_json::to_string_pretty(&value)?;
                    highlighter.print_json(&pretty)?;
                    println!(); // Blank line between entries
                }
                Err(e) => {
                    // Show the parse error
                    eprintln!("\x1b[91mError at line {}: {}\x1b[0m", line_idx + 1, e);
                    eprintln!("\x1b[2m{line}\x1b[0m");
                    println!();
                }
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let color_mode = ColorMode::from_flags(cli.color, cli.no_color);

    display_transcripts(cli.paths, color_mode, cli.verify)
}
