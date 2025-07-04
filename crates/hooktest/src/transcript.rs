use crate::color::{ColorMode, JsonHighlighter};
use anyhow::Result;
use std::fs;
use tenx_hooks::transcript::{format_json_line_for_debug, parse_transcript_with_context};

pub fn display_transcript(path: String, color_mode: ColorMode) -> Result<()> {
    let content = fs::read_to_string(&path)?;
    let parse_result = parse_transcript_with_context(&content);

    let highlighter = JsonHighlighter::new(color_mode);

    // If there are parsing errors, only show those
    if !parse_result.errors.is_empty() {
        // Only show the first error
        let error = &parse_result.errors[0];

        eprintln!(
            "\x1b[91mError at line {}: {}\x1b[0m",
            error.line_number, error.json_error
        );

        eprintln!("\nRaw line content:");
        eprintln!("\x1b[2m{}\x1b[0m", error.line_content);

        eprintln!("\nFormatted for debugging:");
        let formatted = format_json_line_for_debug(&error.line_content);
        highlighter.print_json(&formatted)?;

        // Try to provide more context about the error
        let column = error.json_error.column();
        eprintln!("\nError location (column {column})");
        if column > 0 {
            let pointer = " ".repeat(column.saturating_sub(1)) + "^";
            eprintln!("\x1b[93m{pointer}\x1b[0m");
        }

        // Exit with error code
        std::process::exit(1);
    }

    for (i, entry) in parse_result.entries.iter().enumerate() {
        if i > 0 {
            println!();
        }
        let json = entry.to_debug_json()?;
        highlighter.print_json(&json)?;
    }

    Ok(())
}
