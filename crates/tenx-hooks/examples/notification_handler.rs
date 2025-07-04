use tenx_hooks::{HookResponse, Input, Notification, Result};

fn main() -> Result<()> {
    // Read the hook input from stdin
    let notification = Notification::read()?;

    // Log notification info to stderr (visible in hooktest output)
    eprintln!("Notification received!");
    eprintln!("Hook event name: {}", notification.hook_event_name);
    eprintln!("Message: {}", notification.message);
    eprintln!("Session ID: {}", notification.session_id);

    // Check if the notification mentions certain keywords
    let message_lower = notification.message.to_lowercase();

    if message_lower.contains("danger") || message_lower.contains("destructive") {
        // Stop Claude from continuing due to dangerous operation
        notification
            .stop("Stopped due to potentially dangerous operation. Please review carefully.")
            .respond();
    }

    if message_lower.contains("production") || message_lower.contains("live") {
        // Stop for production-related operations
        notification
            .stop("Stopped: Production environment detected. Manual approval required.")
            .respond();
    }

    // For all other notifications, let Claude continue normally
    Notification::passthrough().respond();
}
