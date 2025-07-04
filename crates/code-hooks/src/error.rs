use std::io;
use thiserror::Error;

/// Error types for hook operations
#[derive(Debug, Error)]
pub enum Error {
    /// Error reading from stdin
    #[error("failed to read from stdin: {0}")]
    Io(#[from] io::Error),

    /// Error parsing JSON input
    #[error("failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// Invalid exit code provided
    #[error("invalid exit code {0}: codes 0 and 2 are reserved")]
    InvalidExitCode(i32),
}

/// Type alias for Results in this library
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn test_error_display() {
        let io_err = Error::Io(io::Error::other("test"));
        assert_eq!(io_err.to_string(), "failed to read from stdin: test");

        let json_err = Error::JsonParse(serde_json::from_str::<Value>("invalid").unwrap_err());
        assert!(json_err.to_string().contains("failed to parse JSON"));

        let exit_err = Error::InvalidExitCode(0);
        assert_eq!(
            exit_err.to_string(),
            "invalid exit code 0: codes 0 and 2 are reserved"
        );
    }
}
