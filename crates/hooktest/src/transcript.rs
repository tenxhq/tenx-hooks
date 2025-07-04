use anyhow::Result;
use std::fs;
use std::io::Write;
use tenx_hooks::transcript::{format_json_line_for_debug, parse_transcript_with_context};
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub fn display_transcript(path: String) -> Result<()> {
    let content = fs::read_to_string(&path)?;
    let parse_result = parse_transcript_with_context(&content);

    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    // If there are parsing errors, only show those
    if !parse_result.errors.is_empty() {
        // Only show the first error
        let error = &parse_result.errors[0];

        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
        writeln!(
            stdout,
            "Error at line {}: {}",
            error.line_number, error.json_error
        )?;
        stdout.reset()?;

        writeln!(stdout, "\nRaw line content:")?;
        stdout.set_color(ColorSpec::new().set_dimmed(true))?;
        writeln!(stdout, "{}", error.line_content)?;
        stdout.reset()?;

        writeln!(stdout, "\nFormatted for debugging:")?;
        let formatted = format_json_line_for_debug(&error.line_content);
        highlight_json(&mut stdout, &formatted)?;

        // Try to provide more context about the error
        let column = error.json_error.column();
        writeln!(stdout, "\nError location (column {})", column)?;
        if column > 0 {
            let pointer = " ".repeat(column.saturating_sub(1)) + "^";
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
            writeln!(stdout, "{}", pointer)?;
            stdout.reset()?;
        }

        // Exit with error code
        std::process::exit(1);
    }

    // Only show successfully parsed entries if there were no errors
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
    writeln!(stdout, "=== TRANSCRIPT PARSING SUMMARY ===")?;
    stdout.reset()?;
    writeln!(stdout, "Total lines: {}", content.lines().count())?;
    writeln!(
        stdout,
        "Successfully parsed entries: {}",
        parse_result.entries.len()
    )?;
    writeln!(stdout)?;

    for (i, entry) in parse_result.entries.iter().enumerate() {
        if i > 0 {
            writeln!(stdout)?;
        }

        // Show entry summary
        stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true))?;
        writeln!(stdout, "Entry {}: {}", i + 1, entry.description())?;
        stdout.reset()?;

        // Convert entry to indented JSON
        let json = entry.to_debug_json()?;

        // Syntax highlight the JSON
        highlight_json(&mut stdout, &json)?;
    }

    Ok(())
}

fn highlight_json(stdout: &mut StandardStream, json: &str) -> Result<()> {
    let mut chars = json.chars().peekable();
    let mut in_string = false;
    let mut escape_next = false;

    while let Some(ch) = chars.next() {
        if escape_next {
            write!(stdout, "{ch}")?;
            escape_next = false;
            continue;
        }

        match ch {
            '"' => {
                if in_string {
                    write!(stdout, "{ch}")?;
                    stdout.reset()?;
                    in_string = false;
                } else {
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                    write!(stdout, "{ch}")?;
                    in_string = true;
                }
            }
            '\\' if in_string => {
                write!(stdout, "{ch}")?;
                escape_next = true;
            }
            ':' if !in_string => {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
                write!(stdout, "{ch}")?;
                stdout.reset()?;
            }
            '{' | '}' | '[' | ']' if !in_string => {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
                write!(stdout, "{ch}")?;
                stdout.reset()?;
            }
            ',' if !in_string => {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
                write!(stdout, "{ch}")?;
                stdout.reset()?;
            }
            _ if !in_string
                && (ch.is_numeric() || ch == '-' || ch == '.' || ch == 'e' || ch == 'E') =>
            {
                // Likely part of a number
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Cyan)))?;
                write!(stdout, "{ch}")?;

                // Continue reading the number
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_numeric()
                        || next_ch == '.'
                        || next_ch == 'e'
                        || next_ch == 'E'
                        || next_ch == '-'
                        || next_ch == '+'
                    {
                        write!(stdout, "{}", chars.next().unwrap())?;
                    } else {
                        break;
                    }
                }
                stdout.reset()?;
            }
            't' | 'f' | 'n' if !in_string => {
                // Check for true, false, null
                let mut word = String::from(ch);
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_alphabetic() {
                        word.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                if word == "true" || word == "false" || word == "null" {
                    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
                    write!(stdout, "{word}")?;
                    stdout.reset()?;
                } else {
                    write!(stdout, "{word}")?;
                }
            }
            _ => {
                write!(stdout, "{ch}")?;
            }
        }
    }

    writeln!(stdout)?;
    stdout.reset()?;
    Ok(())
}
