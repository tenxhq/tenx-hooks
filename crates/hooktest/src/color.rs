use anyhow::Result;
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
