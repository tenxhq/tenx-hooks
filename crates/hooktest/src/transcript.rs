use crate::color::{ColorMode, JsonHighlighter};
use anyhow::Result;
use std::fs;
use tenx_hooks::transcript::{TranscriptEntry, parse_transcript_with_context};

pub fn display_transcript(path: String, color_mode: ColorMode, strict: bool) -> Result<()> {
    let content = fs::read_to_string(&path)?;
    let highlighter = JsonHighlighter::new(color_mode);

    if strict {
        // Use the context parsing for detailed error information
        let parse_result = parse_transcript_with_context(&content);

        // If there are parsing errors, show those first
        if !parse_result.errors.is_empty() {
            for error in &parse_result.errors {
                eprintln!(
                    "\x1b[91mError at line {}: {}\x1b[0m",
                    error.line_number, error.json_error
                );

                eprintln!("\nRaw line content:");
                eprintln!("\x1b[2m{}\x1b[0m", error.line_content);

                // Try to pretty-print the line if it's partial JSON
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&error.line_content) {
                    eprintln!("\nFormatted:");
                    let formatted = serde_json::to_string_pretty(&value)?;
                    highlighter.print_json(&formatted)?;
                }

                let column = error.json_error.column();
                if column > 0 {
                    eprintln!("\nError location (column {column})");
                    let pointer = " ".repeat(column.saturating_sub(1)) + "^";
                    eprintln!("\x1b[93m{pointer}\x1b[0m");
                }
                eprintln!(); // Add blank line between errors
            }

            // Exit with error code if there were parsing errors
            std::process::exit(1);
        }

        // Display successfully parsed entries with their descriptions
        println!(
            "\x1b[92mSuccessfully parsed {} entries\x1b[0m",
            parse_result.entries.len()
        );

        for (idx, _entry) in parse_result.entries.iter().enumerate() {
            println!("\x1b[94m[{}]\x1b[0m Entry #{}", idx + 1, idx + 1);
        }
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

#[allow(dead_code)]
pub fn validate_transcript(path: String) -> Result<()> {
    let content = fs::read_to_string(&path)?;
    let parse_result = parse_transcript_with_context(&content);

    if parse_result.errors.is_empty() {
        println!(
            "\x1b[92m✓ All {} entries parsed successfully\x1b[0m",
            parse_result.entries.len()
        );
        Ok(())
    } else {
        for error in &parse_result.errors {
            eprintln!(
                "\x1b[91mError at line {}: {}\x1b[0m",
                error.line_number, error.json_error
            );
            eprintln!("Line content: {}", error.line_content);
        }
        anyhow::bail!("{} parsing errors found", parse_result.errors.len())
    }
}

#[allow(dead_code)]
pub fn print_entry_for_debugging(entry: &TranscriptEntry) -> Result<()> {
    // Serialize the entry to JSON for debugging
    let json = serde_json::to_string_pretty(entry)?;
    let highlighter = JsonHighlighter::new(ColorMode::Auto);
    highlighter.print_json(&json)?;
    Ok(())
}
