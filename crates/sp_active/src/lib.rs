//! The weekly active count (ADR 0095, rolling since ADR 0121), in place of web-backend's
//! `weekly_active_users_global`.
//!
//! A meaningful action (a vote, post, comment, role taken) writes an `Active` link from its
//! day's path to its author, signed by them, at most one an hour. Weekly active at any moment
//! is the number of distinct people (not bots, ADR 0118) with a link in the seven days before
//! it, so it moves as people act and as their last action ages out (owner, 2026-11-25). Any
//! reader counts the same links (full nodes hold them all), so votes snapshot the count when
//! they open (ADR 0033) and ADR 0040's challenge window covers an under-count.
//! Pure: no HDK, no I/O.

use std::collections::BTreeSet;
use std::ops::RangeInclusive;

const DAY_US: i64 = 24 * 60 * 60 * 1_000_000;
pub const WEEK_US: i64 = 7 * DAY_US;
/// The rolling window weekly active counts over.
pub const WINDOW_US: i64 = WEEK_US;
/// A person writes at most one `Active` link an hour, so the count is right to the hour.
pub const RECORD_GAP_US: i64 = 60 * 60 * 1_000_000;
/// 1970-01-01 was a Thursday; weeks start on Mondays (00:00 UTC), as ISO weeks do.
const MONDAY_OFFSET_US: i64 = 4 * DAY_US;

/// The start of the week holding `at` (microseconds since the epoch): the moderator board
/// still settles weekly (`sp_board`).
#[must_use]
pub fn week_start(at: i64) -> i64 {
    (at - MONDAY_OFFSET_US).div_euclid(WEEK_US) * WEEK_US + MONDAY_OFFSET_US
}

/// The UTC day of `at` (days since the epoch): an `Active` path's name is `active.day.<day>`.
#[must_use]
pub fn day_of(at: i64) -> i64 {
    at.div_euclid(DAY_US)
}

/// The days whose paths hold every link in the window before `at`.
#[must_use]
pub fn window_days(at: i64) -> RangeInclusive<i64> {
    day_of(at - WINDOW_US)..=day_of(at)
}

/// Weekly active at `at`: distinct authors with a link in `(at - WINDOW_US, at]`, each link
/// counting only if `counts(author, written)` held (not a bot then).
pub fn active_at<A: Ord>(
    at: i64,
    links: impl IntoIterator<Item = (A, i64)>,
    counts: impl Fn(&A, i64) -> bool,
) -> u32 {
    let people: BTreeSet<A> = links
        .into_iter()
        .filter(|(who, t)| *t > at - WINDOW_US && *t <= at && counts(who, *t))
        .map(|(who, _)| who)
        .collect();
    u32::try_from(people.len()).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2026-11-16 00:00 UTC is a Monday.
    const MONDAY: i64 = 1_794_787_200 * 1_000_000;
    const HOUR: i64 = RECORD_GAP_US;

    #[test]
    fn weeks_start_on_monday() {
        assert_eq!(week_start(MONDAY), MONDAY);
        assert_eq!(week_start(MONDAY + 6 * DAY_US + 1), MONDAY);
        assert_eq!(week_start(MONDAY - 1), MONDAY - WEEK_US);
    }

    #[test]
    fn the_window_spans_eight_day_paths() {
        let at = MONDAY + 10 * HOUR;
        let days = window_days(at);
        assert_eq!(days.end() - days.start(), 7);
        assert_eq!(*days.end(), day_of(at));
    }

    #[test]
    fn counts_the_seven_days_before_the_moment() {
        let at = MONDAY + 12 * HOUR;
        let links = [
            ("ann", at - HOUR),
            ("ann", at - 2 * HOUR),
            // Exactly seven days ago has just aged out; a second later is still in.
            ("old", at - WINDOW_US),
            ("bob", at - WINDOW_US + 1),
            ("bot", at - HOUR),
            // After the moment: not yet.
            ("cat", at + 1),
        ];
        assert_eq!(active_at(at, links, |who, _| *who != "bot"), 2);
        assert_eq!(active_at(at + 1, links, |who, _| *who != "bot"), 2);
        // An hour on, bob and old have aged out and cat has come in.
        assert_eq!(active_at(at + HOUR, links, |_, _| true), 3);
    }
}
