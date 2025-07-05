# claude-transcript

Parse and analyze Claude conversation transcripts. Provides type-safe structures for working with Claude Code's JSONL transcript format.

## Installation

```toml
[dependencies]
claude-transcript = "0.0.1"
```

## Usage

```rust
use claude_transcript::parse::parse_transcript_with_context;

fn main() {
    let content = std::fs::read_to_string("transcript.json").unwrap();
    let result = parse_transcript_with_context(&content);
    
    if !result.errors.is_empty() {
        for error in result.errors {
            eprintln!("Line {}: {}", error.line_number, error.json_error);
        }
    }
    
    println!("Parsed {} entries", result.entries.len());
}
```

## ttest Example

View and validate transcripts:

```bash
# Pretty-print a transcript
cargo run --example ttest -- transcript.json

# Validate format (silent unless errors)
cargo run --example ttest -- --verify transcript.json
```

## License

MIT