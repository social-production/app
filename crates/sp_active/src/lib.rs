//! The weekly active count (ADR 0095), in place of web-backend's `weekly_active_users_global`.
//!
//! The first meaningful action of each week (a vote, post, comment, role taken) writes one
//! `Active` link from the week's path to its author, signed by them. Weekly active is the
//! number of distinct people with Full standing who wrote one for the last complete week.
//! Any reader counts the same links (full nodes hold them all), so votes snapshot the count
//! when they open (ADR 0033) and ADR 0040's challenge window covers an under-count.
//! Pure: no HDK, no I/O.

use std::collections::BTreeSet;

const DAY_US: i64 = 24 * 60 * 60 * 1_000_000;
pub const WEEK_US: i64 = 7 * DAY_US;
/// 1970-01-01 was a Thursday; weeks start on Mondays (00:00 UTC), as ISO weeks do.
const MONDAY_OFFSET_US: i64 = 4 * DAY_US;

/// The start of the week holding `at` (microseconds since the epoch).
#[must_use]
pub fn week_start(at: i64) -> i64 {
    (at - MONDAY_OFFSET_US).div_euclid(WEEK_US) * WEEK_US + MONDAY_OFFSET_US
}

/// The week number of `at` (weeks since the Monday before the epoch): the `Active` path's
/// name is `active.<week>`.
#[must_use]
pub fn week_of(at: i64) -> i64 {
    (at - MONDAY_OFFSET_US).div_euclid(WEEK_US)
}

/// The last complete week before `at`, which a vote opening at `at` counts. Devices that were
/// offline write their link when they next start, so the current week is never final.
#[must_use]
pub fn counted_week(at: i64) -> i64 {
    week_of(at) - 1
}

/// The weekly active count from one week's `Active` links: distinct authors, each counted
/// only if `is_full(author, at)` held when they wrote it, and only links written in that week
/// or later (an `Active` link can't be backdated past its own timestamp).
pub fn weekly_active<A: Ord>(
    week: i64,
    links: impl IntoIterator<Item = (A, i64)>,
    is_full: impl Fn(&A, i64) -> bool,
) -> u32 {
    let start = MONDAY_OFFSET_US + week * WEEK_US;
    let people: BTreeSet<A> = links
        .into_iter()
        .filter(|(who, at)| *at >= start && is_full(who, *at))
        .map(|(who, _)| who)
        .collect();
    u32::try_from(people.len()).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 2026-11-16 00:00 UTC is a Monday.
    const MONDAY: i64 = 1_794_787_200 * 1_000_000;

    #[test]
    fn weeks_start_on_monday() {
        assert_eq!(week_start(MONDAY), MONDAY);
        assert_eq!(week_start(MONDAY + 6 * DAY_US + 1), MONDAY);
        assert_eq!(week_start(MONDAY - 1), MONDAY - WEEK_US);
        assert_eq!(week_of(MONDAY) - week_of(MONDAY - 1), 1);
        assert_eq!(MONDAY_OFFSET_US + week_of(MONDAY) * WEEK_US, MONDAY);
    }

    #[test]
    fn a_vote_counts_the_last_complete_week() {
        assert_eq!(counted_week(MONDAY + DAY_US), week_of(MONDAY) - 1);
        assert_eq!(counted_week(MONDAY), week_of(MONDAY - 1));
    }

    #[test]
    fn counts_distinct_full_members_and_drops_backdated_links() {
        let week = week_of(MONDAY);
        let links = [
            ("ann", MONDAY + 1),
            ("ann", MONDAY + 2),
            ("bob", MONDAY + DAY_US),
            ("new", MONDAY + DAY_US),
            ("old", MONDAY - 1),
            // Written late (offline device): still counts.
            ("cat", MONDAY + 9 * DAY_US),
        ];
        let n = weekly_active(week, links, |who, _| *who != "new");
        assert_eq!(n, 3);
    }
}
