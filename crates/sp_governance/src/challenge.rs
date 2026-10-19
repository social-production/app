//! The challenge window for under-counted populations (docs/GOVERNANCE.md, ADR 0040).
//!
//! A proposal's population N is its proposer's snapshot of members (ADR 0033). Leaving members
//! out makes N smaller and a decision easier to reach. Any member may publish a challenge citing
//! memberships from before the proposal that the snapshot lacks, while the proposal is open or
//! within [`WINDOW_MICROS`] of its first Outcome. Counted challenges enlarge N; an Outcome
//! decides the proposal only if it cites every counted challenge, so a challenged decision
//! stops applying and the proposal reopens until one that does is written.

use std::collections::BTreeSet;

/// 48 hours, in microseconds (Holochain timestamps).
pub const WINDOW_MICROS: i64 = 48 * 60 * 60 * 1_000_000;

/// When the challenge window after a proposal's first Outcome closes.
#[must_use]
pub fn window_end(first_outcome: i64) -> i64 {
    first_outcome.saturating_add(WINDOW_MICROS)
}

/// Whether a challenge made at `at` counts: while there is no Outcome yet, or up to the end of
/// the window after the first one.
#[must_use]
pub fn in_time(first_outcome: Option<i64>, at: i64) -> bool {
    first_outcome.is_none_or(|first| at <= window_end(first))
}

/// The members challenges add: every challenged member the snapshot lacks, each once.
#[must_use]
pub fn added<K: Ord + Clone>(
    snapshot: &BTreeSet<K>,
    challenged: impl IntoIterator<Item = K>,
) -> BTreeSet<K> {
    challenged
        .into_iter()
        .filter(|k| !snapshot.contains(k))
        .collect()
}

/// The Outcome that decides a proposal: the earliest (by time, then position) whose cited
/// challenges include every counted one. `None` means the proposal is open again.
#[must_use]
pub fn deciding<H: Ord>(outcomes: &[(i64, BTreeSet<H>)], counted: &BTreeSet<H>) -> Option<usize> {
    outcomes
        .iter()
        .enumerate()
        .filter(|(_, (_, cited))| counted.is_subset(cited))
        .min_by_key(|(i, (at, _))| (*at, *i))
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HOUR: i64 = 60 * 60 * 1_000_000;

    fn set(items: &[u8]) -> BTreeSet<u8> {
        items.iter().copied().collect()
    }

    #[test]
    fn the_window_is_48_hours_after_the_first_outcome() {
        assert!(in_time(None, 1_000 * HOUR), "open proposals take challenges");
        assert!(in_time(Some(10 * HOUR), 58 * HOUR));
        assert!(!in_time(Some(10 * HOUR), 58 * HOUR + 1));
        assert_eq!(window_end(i64::MAX), i64::MAX, "no overflow");
    }

    #[test]
    fn challenges_add_each_missing_member_once() {
        let snapshot = set(&[1, 2]);
        assert_eq!(added(&snapshot, [2, 3, 3, 4]), set(&[3, 4]));
        assert!(added(&snapshot, [1]).is_empty());
    }

    #[test]
    fn without_challenges_the_first_outcome_decides() {
        let outcomes = [(5, set(&[])), (3, set(&[]))];
        assert_eq!(deciding(&outcomes, &set(&[])), Some(1));
        assert_eq!(deciding::<u8>(&[], &set(&[])), None);
    }

    #[test]
    fn a_counted_challenge_voids_outcomes_that_ignore_it() {
        let outcomes = [(1, set(&[])), (4, set(&[7, 9])), (2, set(&[7]))];
        assert_eq!(deciding(&outcomes, &set(&[7])), Some(2));
        assert_eq!(deciding(&outcomes, &set(&[7, 9])), Some(1));
        assert_eq!(deciding(&outcomes, &set(&[8])), None, "reopened");
    }
}
