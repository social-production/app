//! Platform moderators (ADR 0095), ported from web-backend's `services/board.py`.
//!
//! Anyone may volunteer. Full members vote yes or no on each candidate or moderator; votes
//! persist. A candidate with at least the quorum of votes and 66% yes becomes a moderator. A
//! moderator whose approval falls below 66% is removed at once; one whose vote count falls
//! below the quorum gets a one-week grace period, then is removed.
//!
//! Nothing is stored as a state: every reader replays the same candidacies and votes in time
//! order (`replay`). The quorum comes from the caller (web: `required_votes(weekly active)`),
//! as a function of time. Pure: no HDK, no I/O.

mod replay;

pub use replay::{replay, Act, Kind, Standing, State, Stats};

/// Share of counted votes that must say yes, in whole percent (web's `MIN_APPROVAL_RATIO`).
pub const APPROVAL_PERCENT: u32 = 66;

const DAY_US: i64 = 24 * 60 * 60 * 1_000_000;
/// A moderator short of the quorum keeps the role this long (web's `GRACE_PERIOD_DAYS`).
pub const GRACE_US: i64 = 7 * DAY_US;
pub use sp_active::{week_start, WEEK_US};

#[cfg(test)]
mod tests;
