use serde::{Deserialize, Serialize};

/// Helper function for serde to skip serializing None values
pub(crate) fn is_none<T>(opt: &Option<T>) -> bool {
    opt.is_none()
}

/// Decision type for approve/block operations.
///
/// Used in PreToolUse, PostToolUse, and Stop hooks to control execution flow.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    /// Approve the operation (PreToolUse only - bypasses permission system)
    Approve,
    /// Block the operation and provide feedback to Claude
    Block,
}
