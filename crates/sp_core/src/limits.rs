//! Size limits enforced in integrity validation. Changing one changes the DNA hash of
//! every DNA that validates against it, so treat these as part of the network contract.

/// Short single-line text (titles, names).
pub const SHORT_TEXT_MAX: usize = 200;
/// Health-check notes and other tiny free text.
pub const NOTE_MAX: usize = 280;
/// Long free text (descriptions, posts).
pub const LONG_TEXT_MAX: usize = 20_000;

/// True when `text` is non-empty after trimming and at most `max` characters.
#[must_use]
pub fn text_ok(text: &str, max: usize) -> bool {
    !text.trim().is_empty() && text.chars().count() <= max
}

#[cfg(test)]
mod tests {
    use super::{text_ok, NOTE_MAX};

    #[test]
    fn rejects_blank_and_oversized() {
        assert!(text_ok("hi", NOTE_MAX));
        assert!(!text_ok("   ", NOTE_MAX));
        assert!(!text_ok(&"x".repeat(NOTE_MAX + 1), NOTE_MAX));
    }
}
