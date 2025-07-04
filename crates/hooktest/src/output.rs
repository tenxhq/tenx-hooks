use crate::color::{ColorMode, JsonHighlighter};
use anyhow::Result;
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct Output {
    stdout: StandardStream,
    json_highlighter: JsonHighlighter,
}

impl Output {
    pub fn new(color_mode: ColorMode) -> Self {
        let color_choice = if color_mode.should_colorize() {
            ColorChoice::Always
        } else {
            ColorChoice::Never
        };
        Self {
            stdout: StandardStream::stdout(color_choice),
            json_highlighter: JsonHighlighter::new(color_mode),
        }
    }

    /// Print a level 1 heading in cyan
    pub fn h1(&mut self, text: &str) -> Result<()> {
        self.stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Cyan)).set_bold(true))?;
        writeln!(self.stdout, "\n=== {text} ===")?;
        self.stdout.reset()?;
        Ok(())
    }

    /// Print a block of text with optional color
    pub fn block(&mut self, text: &str) -> Result<()> {
        writeln!(self.stdout, "{text}")?;
        Ok(())
    }

    /// Print colored text
    pub fn color(&mut self, text: &str, color: Color, bold: bool) -> Result<()> {
        let mut spec = ColorSpec::new();
        spec.set_fg(Some(color));
        if bold {
            spec.set_bold(true);
        }
        self.stdout.set_color(&spec)?;
        write!(self.stdout, "{text}")?;
        self.stdout.reset()?;
        Ok(())
    }

    /// Print colored text with newline
    #[allow(dead_code)]
    pub fn color_line(&mut self, text: &str, color: Color, bold: bool) -> Result<()> {
        self.color(text, color, bold)?;
        writeln!(self.stdout)?;
        Ok(())
    }

    /// Print a label (yellow) followed by value
    pub fn label(&mut self, label: &str, value: &str) -> Result<()> {
        self.stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Yellow)))?;
        write!(self.stdout, "{label}: ")?;
        self.stdout.reset()?;
        writeln!(self.stdout, "{value}")?;
        Ok(())
    }

    /// Print success text in green
    pub fn success(&mut self, text: &str) -> Result<()> {
        self.color(text, Color::Green, true)
    }

    /// Print error text in red
    pub fn error(&mut self, text: &str) -> Result<()> {
        self.color(text, Color::Red, true)
    }

    /// Print JSON with syntax highlighting
    pub fn json(&mut self, json: &serde_json::Value) -> Result<()> {
        let json_str = serde_json::to_string_pretty(json)?;
        self.json_highlighter.print_json(&json_str)?;
        Ok(())
    }

    /// Print dimmed text
    pub fn dimmed(&mut self, text: &str) -> Result<()> {
        self.stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::White)).set_dimmed(true))?;
        writeln!(self.stdout, "{text}")?;
        self.stdout.reset()?;
        Ok(())
    }

    /// Write directly to stdout
    pub fn write(&mut self, text: &str) -> Result<()> {
        write!(self.stdout, "{text}")?;
        Ok(())
    }

    /// Write line directly to stdout
    #[allow(dead_code)]
    pub fn writeln(&mut self, text: &str) -> Result<()> {
        writeln!(self.stdout, "{text}")?;
        Ok(())
    }

    /// New line
    pub fn newline(&mut self) -> Result<()> {
        writeln!(self.stdout)?;
        Ok(())
    }
}
