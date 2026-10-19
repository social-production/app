//! Project chat messages (web's project chat comments, ADR 0038). Shared by the discussion
//! integrity zome (validation) and its coordinator (input checks before writing).

use crate::limits::text_ok;

/// web stores chat bodies as unbounded text; a cap keeps one message from filling a DHT op.
/// Long enough for any chat message, short of a post (`LONG_TEXT_MAX`).
pub const MESSAGE_MAX: usize = 4_000;
/// How many characters of the original a reply quote shows (web's `replyPreview`).
pub const REPLY_PREVIEW_MAX: usize = 120;
/// How many of the newest messages a chat read returns (web's comment page size).
pub const PAGE_MAX: usize = 200;

/// A message body: some text after trimming, at most `MESSAGE_MAX` characters.
pub fn check_message(body: &str) -> Result<(), String> {
    if text_ok(body, MESSAGE_MAX) {
        Ok(())
    } else {
        Err(format!("a message is 1–{MESSAGE_MAX} characters"))
    }
}

/// The start of a message, as a reply quote shows it.
#[must_use]
pub fn preview(body: &str) -> String {
    body.chars().take(REPLY_PREVIEW_MAX).collect()
}

#[cfg(test)]
mod tests {
    use super::{check_message, preview, MESSAGE_MAX, REPLY_PREVIEW_MAX};

    #[test]
    fn messages_need_text_within_the_cap() {
        assert!(check_message("hello").is_ok());
        assert!(check_message("  \n ").is_err());
        assert!(check_message(&"é".repeat(MESSAGE_MAX)).is_ok());
        assert!(check_message(&"x".repeat(MESSAGE_MAX + 1)).is_err());
    }

    #[test]
    fn previews_cut_by_characters() {
        assert_eq!(preview("short"), "short");
        assert_eq!(preview(&"ü".repeat(300)).chars().count(), REPLY_PREVIEW_MAX);
    }
}
