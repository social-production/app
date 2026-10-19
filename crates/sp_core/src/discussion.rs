//! Project discussion threads, their comments and votes (web's threads, comments and
//! content votes; ADR 0041). Shared by the discussion integrity zome (validation) and its
//! coordinator (input checks before writing, vote tallies).

use crate::chat::MESSAGE_MAX;
use crate::limits::{text_ok, LONG_TEXT_MAX, SHORT_TEXT_MAX};

/// web's `threads.title` is `varchar(200)`.
pub const TITLE_MAX: usize = SHORT_TEXT_MAX;
/// web's opening post is unbounded text; this is the app's long-text cap.
pub const BODY_MAX: usize = LONG_TEXT_MAX;
/// web keeps thread comments in the same `comments` table as chat, so they share its cap.
pub const COMMENT_MAX: usize = MESSAGE_MAX;
/// web's `get_comments` returns at most 200 comments, oldest first.
pub const COMMENT_PAGE_MAX: usize = 200;
/// How many threads one list read returns (the newest ones, before sorting).
pub const THREAD_PAGE_MAX: usize = 200;

/// Link tags for a vote (web's `content_votes.direction`: up = 1, down = -1).
pub const VOTE_UP: u8 = 1;
pub const VOTE_DOWN: u8 = 2;

/// A new thread: a title of 1–200 characters and an opening post of 1–20,000.
pub fn check_thread(title: &str, body: &str) -> Result<(), String> {
    if !text_ok(title, TITLE_MAX) {
        return Err(format!("a thread title is 1–{TITLE_MAX} characters"));
    }
    if !text_ok(body, BODY_MAX) {
        return Err(format!("an opening post is 1–{BODY_MAX} characters"));
    }
    Ok(())
}

/// A comment body: some text after trimming, at most `COMMENT_MAX` characters.
pub fn check_comment(body: &str) -> Result<(), String> {
    if text_ok(body, COMMENT_MAX) {
        Ok(())
    } else {
        Err(format!("a comment is 1–{COMMENT_MAX} characters"))
    }
}

/// The link tag for a vote direction (web's up = 1, down = -1); 0 ("neutral") has none.
#[must_use]
pub fn vote_tag(direction: i8) -> Option<u8> {
    match direction {
        1 => Some(VOTE_UP),
        -1 => Some(VOTE_DOWN),
        _ => None,
    }
}

/// The direction a vote link's tag carries; anything else counts for nothing.
#[must_use]
pub fn vote_direction(tag: &[u8]) -> i8 {
    match tag {
        [VOTE_UP] => 1,
        [VOTE_DOWN] => -1,
        _ => 0,
    }
}

/// A target's score: ups minus downs (web's `vote_count`).
#[must_use]
pub fn tally(directions: impl IntoIterator<Item = i8>) -> i32 {
    directions.into_iter().map(i32::from).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threads_need_a_title_and_an_opening_post() {
        assert!(check_thread("Where to plant?", "Sun or shade?").is_ok());
        assert!(check_thread("  ", "Body").is_err());
        assert!(check_thread("Title", " \n ").is_err());
        assert!(check_thread(&"t".repeat(TITLE_MAX + 1), "Body").is_err());
        assert!(check_thread(&"é".repeat(TITLE_MAX), &"b".repeat(BODY_MAX)).is_ok());
        assert!(check_thread("Title", &"b".repeat(BODY_MAX + 1)).is_err());
    }

    #[test]
    fn comments_need_text_within_the_cap() {
        assert!(check_comment("Agreed").is_ok());
        assert!(check_comment("   ").is_err());
        assert!(check_comment(&"x".repeat(COMMENT_MAX + 1)).is_err());
    }

    #[test]
    fn votes_round_trip_through_tags() {
        assert_eq!(vote_tag(1).map(|t| vote_direction(&[t])), Some(1));
        assert_eq!(vote_tag(-1).map(|t| vote_direction(&[t])), Some(-1));
        assert_eq!(vote_tag(0), None);
        assert_eq!(vote_tag(5), None);
        assert_eq!(vote_direction(&[]), 0);
        assert_eq!(vote_direction(&[VOTE_UP, VOTE_UP]), 0);
    }

    #[test]
    fn tallies_net_ups_and_downs() {
        assert_eq!(tally([1, 1, -1]), 1);
        assert_eq!(tally([-1, -1]), -2);
        assert_eq!(tally([]), 0);
    }
}
