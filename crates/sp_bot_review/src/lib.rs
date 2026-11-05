//! Public bot reviews (ADR 0074), ported from web-backend's `services/bot_reviews.py`.
//!
//! Bot marks never pause anyone on their own. Three marks from full members open a review that
//! full members vote on (66% "bot" to pass). Upheld: the account is paused. Cleared, or not
//! decided within a week: the marks are removed and those markers can't mark the account again
//! for 30 days. A paused account can ask for one re-review 30 days later.
//!
//! Nothing is stored as a state: every reader replays the same marks, votes and re-review
//! requests in time order (`replay`). Pure: no HDK, no I/O.

mod replay;
mod tally;

pub use replay::{replay, Act, Closed, Kind, Logged, OpenReview, State};
pub use tally::{tally, Tally};

/// Counted marks that open a review (web's `MARKS_TO_OPEN`).
pub const MARKS_TO_OPEN: usize = 3;
/// Never fewer counted votes than this, however few people are involved (web's `MIN_VOTES`).
pub const MIN_VOTES: u32 = 3;
/// Share of counted votes that must say "bot", in whole percent (web's `BOT_SHARE` 0.66).
pub const BOT_SHARE_PERCENT: u32 = 66;

const DAY_US: i64 = 24 * 60 * 60 * 1_000_000;
/// How long a review stays open (web's `REVIEW_DAYS`).
pub const REVIEW_US: i64 = 7 * DAY_US;
/// How long a cleared review's markers wait to mark again (web's `COOLDOWN_DAYS`), and how
/// long the vouchers of an upheld account wait to vouch again.
pub const COOLDOWN_US: i64 = 30 * DAY_US;
/// When a paused account may ask for its one re-review (web's `APPEAL_AFTER_DAYS`).
pub const APPEAL_AFTER_US: i64 = 30 * DAY_US;

/// Link tags. Marks: bot or clear; votes: bot or real.
pub const TAG_CLEAR: u8 = 0;
pub const TAG_BOT: u8 = 1;
pub const TAG_REAL: u8 = 0;

#[cfg(test)]
mod tests;
