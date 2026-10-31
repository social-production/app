//! Threads and posts (ADR 0070): commons spaces whose charter mode is `"thread"` or `"post"`,
//! with no lifecycle. Their comments are the space's chat; comments take up/down votes.
//! Ported from web-backend `services/content/{threads,posts}.py`.

/// web's thread: a titled discussion tagged with channels or communities.
pub const THREAD: &str = "thread";
/// web's post: a short public note on its author's personal feed.
pub const POST: &str = "post";
/// The only subtype either has.
pub const SUBTYPE: &str = "standard";

/// The tag byte of a comment vote link (web's `content_votes.direction` 1 / -1).
pub const UP: u8 = 0;
pub const DOWN: u8 = 1;

/// A thread or a post.
#[must_use]
pub fn is_discussion(mode: &str) -> bool {
    mode == THREAD || mode == POST
}

/// A thread or post charter's mode and subtype.
///
/// # Errors
/// When the mode isn't a thread or post, or the subtype isn't `standard`.
pub fn check_kind(mode: &str, subtype: &str) -> Result<(), String> {
    if is_discussion(mode) && subtype == SUBTYPE {
        Ok(())
    } else {
        Err(format!(
            "a thread or post has mode {THREAD:?} or {POST:?} and subtype {SUBTYPE:?}"
        ))
    }
}

/// A vote link's tag: one byte, up or down.
#[must_use]
pub fn is_vote_tag(tag: &[u8]) -> bool {
    matches!(tag, [UP | DOWN])
}

/// The tag for web's direction (1 up, -1 down); `None` for 0 (no vote).
#[must_use]
pub fn tag_of(direction: i8) -> Option<u8> {
    match direction.signum() {
        1 => Some(UP),
        -1 => Some(DOWN),
        _ => None,
    }
}

/// web's direction for a tag (1 up, -1 down, 0 otherwise).
#[must_use]
pub fn direction_of(tag: &[u8]) -> i8 {
    match tag {
        [UP] => 1,
        [DOWN] => -1,
        _ => 0,
    }
}

/// The title a post's charter carries (posts have none on web): the body's first line, cut to
/// 80 characters as web's search index does.
#[must_use]
pub fn post_title(body: &str) -> String {
    let line = body.trim().lines().next().unwrap_or_default().trim();
    let title: String = line.chars().take(80).collect();
    if title.is_empty() {
        "Post".into()
    } else {
        title
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds() {
        assert!(check_kind(THREAD, SUBTYPE).is_ok());
        assert!(check_kind(POST, SUBTYPE).is_ok());
        assert!(check_kind("event", SUBTYPE).is_err());
        assert!(check_kind(POST, "software").is_err());
    }

    #[test]
    fn votes() {
        assert!(is_vote_tag(&[UP]) && is_vote_tag(&[DOWN]));
        assert!(!is_vote_tag(&[2]) && !is_vote_tag(&[]) && !is_vote_tag(&[0, 0]));
        assert_eq!(tag_of(1), Some(UP));
        assert_eq!(tag_of(-3), Some(DOWN));
        assert_eq!(tag_of(0), None);
        assert_eq!(direction_of(&[DOWN]), -1);
        assert_eq!(direction_of(&[UP]), 1);
    }

    #[test]
    fn titles() {
        assert_eq!(post_title("  Hello\nworld"), "Hello");
        assert_eq!(post_title("   "), "Post");
        assert_eq!(post_title(&"x".repeat(100)).chars().count(), 80);
    }
}
