use crate::color::{ColorMode, JsonHighlighter};
use anyhow::Result;
use std::fs;
use tenx_hooks::transcript::{
    format_json_line_for_debug, parse_transcript_with_context, parse_transcript_with_raw,
    validate_transcript_entry,
};

pub fn display_transcript(path: String, color_mode: ColorMode, strict: bool) -> Result<()> {
    let content = fs::read_to_string(&path)?;

    if strict {
        // Use the raw parsing mode for strict validation
        let parse_result = parse_transcript_with_raw(&content);
        let highlighter = JsonHighlighter::new(color_mode);

        // If there are parsing errors, show those first
        if !parse_result.errors.is_empty() {
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

            let column = error.json_error.column();
            eprintln!("\nError location (column {column})");
            if column > 0 {
                let pointer = " ".repeat(column.saturating_sub(1)) + "^";
                eprintln!("\x1b[93m{pointer}\x1b[0m");
            }

            std::process::exit(1);
        }

        // Validate each entry
        let mut validation_errors = Vec::new();
        for (line_number, entry_with_raw) in parse_result.entries.iter().enumerate() {
            match validate_transcript_entry(entry_with_raw) {
                Ok(missing_fields) => {
                    if !missing_fields.is_empty() {
                        validation_errors.push((line_number + 1, entry_with_raw, missing_fields));
                    }
                }
                Err(e) => {
                    eprintln!(
                        "\x1b[91mValidation error at line {}: {}\x1b[0m",
                        line_number + 1,
                        e
                    );
                    std::process::exit(1);
                }
            }
        }

        // Report validation errors
        if !validation_errors.is_empty() {
            eprintln!("\x1b[91mStrict validation failed!\x1b[0m");
            eprintln!(
                "\nThe following fields are present in the raw transcript but not represented in TranscriptEntry:\n"
            );

            for (line_number, entry_with_raw, missing_fields) in validation_errors {
                eprintln!("\x1b[93mLine {line_number}:\x1b[0m");
                for field in missing_fields {
                    eprintln!("  - {field}");
                }
                eprintln!("\nRaw JSON:");
                let formatted = format_json_line_for_debug(&entry_with_raw.raw);
                highlighter.print_json(&formatted)?;
                eprintln!();
            }

            std::process::exit(1);
        }

        // If validation passes, display normally
        for (i, entry_with_raw) in parse_result.entries.iter().enumerate() {
            if i > 0 {
                println!();
            }
            let json = entry_with_raw.entry.to_debug_json()?;
            highlighter.print_json(&json)?;
        }
    } else {
        // Non-strict mode: use the original logic
        let parse_result = parse_transcript_with_context(&content);
        let highlighter = JsonHighlighter::new(color_mode);

        if !parse_result.errors.is_empty() {
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

            let column = error.json_error.column();
            eprintln!("\nError location (column {column})");
            if column > 0 {
                let pointer = " ".repeat(column.saturating_sub(1)) + "^";
                eprintln!("\x1b[93m{pointer}\x1b[0m");
            }

            std::process::exit(1);
        }

        for (i, entry) in parse_result.entries.iter().enumerate() {
            if i > 0 {
                println!();
            }
            let json = entry.to_debug_json()?;
            highlighter.print_json(&json)?;
        }
    }

    Ok(())
}
