/// Helper functions for exit codes.
///
/// Hooks can communicate status through exit codes as an alternative
/// to JSON output:
/// - Exit code 0: Success, stdout shown in transcript mode
/// - Exit code 2: Blocking error, stderr shown to Claude
/// - Other codes: Non-blocking error, stderr shown to user
use crate::error::{Error, Result};

/// Exit with success (0).
///
/// Stdout will be shown to the user in transcript mode (Ctrl-R).
pub fn success() {
    std::process::exit(0);
}

/// Exit with blocking error (2).
///
/// Stderr will be fed back to Claude to process automatically.
/// - PreToolUse: Blocks the tool call
/// - PostToolUse: Shows error to Claude (tool already ran)
/// - Stop: Blocks stoppage
pub fn block() {
    std::process::exit(2);
}

/// Exit with non-blocking error.
///
/// Stderr is shown to the user and execution continues.
///
/// # Errors
///
/// Returns an error if code is 0 or 2 (reserved exit codes).
pub fn error(code: i32) -> Result<()> {
    if code == 0 || code == 2 {
        return Err(Error::InvalidExitCode(code));
    }
    std::process::exit(code);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_exit_code_validation() {
        // Test that reserved exit codes return errors
        assert!(matches!(error(0), Err(Error::InvalidExitCode(0))));
        assert!(matches!(error(2), Err(Error::InvalidExitCode(2))));
    }
}
