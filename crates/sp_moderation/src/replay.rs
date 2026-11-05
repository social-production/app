//! web-backend's `advance_resolution`, replayed: web moves a report's state at each vote and
//! stores it; here every reader replays the counted votes in time order and gets the same
//! state. A person's later vote replaces their earlier one (web updates the row). Once the
//! report is removed or dismissed, later votes change nothing (web refuses them).

use std::collections::BTreeMap;

use crate::thresholds::Thresholds;
use crate::{Reason, Resolution};

const DAY_MICROS: i64 = 86_400_000_000;

/// One counted vote: when, who, and yes (take it down) or no.
#[derive(Clone, Debug)]
pub struct Ballot<K> {
    pub at: i64,
    pub voter: K,
    pub yes: bool,
}

/// Where the report ended up, with the votes that got it there.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub resolution: Resolution,
    pub yes: u32,
    pub no: u32,
}

/// Replay `ballots` (any order) on a report about an item created at `created_at` (micros).
/// `audience` is the item's eligible audience (author left out) and `score` its popularity,
/// both as read now; each vote is weighed with the item's age at that vote, as on web.
#[must_use]
pub fn replay<K: Ord + Clone>(
    reason: Reason,
    audience: u32,
    score: u32,
    created_at: i64,
    ballots: &[Ballot<K>],
) -> Outcome {
    run(reason, created_at, ballots, |age| {
        Thresholds::at(reason, audience, age, score)
    })
}

/// `replay` for a direct or group chat message (`Thresholds::chat`, web's 1:1 path).
#[must_use]
pub fn replay_chat<K: Ord + Clone>(
    reason: Reason,
    audience: u32,
    created_at: i64,
    ballots: &[Ballot<K>],
) -> Outcome {
    run(reason, created_at, ballots, |age| {
        Thresholds::chat(reason, audience, age)
    })
}

fn run<K: Ord + Clone>(
    reason: Reason,
    created_at: i64,
    ballots: &[Ballot<K>],
    bars: impl Fn(u32) -> Thresholds,
) -> Outcome {
    let mut sorted: Vec<&Ballot<K>> = ballots.iter().collect();
    sorted.sort_by_key(|b| b.at);
    let mut votes: BTreeMap<K, bool> = BTreeMap::new();
    let mut state = Resolution::Open;
    for ballot in sorted {
        votes.insert(ballot.voter.clone(), ballot.yes);
        let (yes, no) = counts(&votes);
        let t = bars(age_days(created_at, ballot.at));
        state = advance(reason, state, yes, no, &t);
        if !state.active() {
            return Outcome {
                resolution: state,
                yes,
                no,
            };
        }
    }
    let (yes, no) = counts(&votes);
    Outcome {
        resolution: state,
        yes,
        no,
    }
}

fn counts<K>(votes: &BTreeMap<K, bool>) -> (u32, u32) {
    let yes = votes.values().filter(|v| **v).count();
    let no = votes.len() - yes;
    (to_u32(yes), to_u32(no))
}

fn to_u32(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

/// Whole days between creation and `at` (web's `age_days < 1` is day 0).
#[must_use]
pub fn age_days(created_at: i64, at: i64) -> u32 {
    u32::try_from((at - created_at).max(0) / DAY_MICROS).unwrap_or(u32::MAX)
}

/// web's `advance_resolution`: from `open` a report lands on `under_review` first; serious
/// harm may go on to `hidden` in the same step; deletion takes a later vote, except in a
/// 1:1 chat.
fn advance(reason: Reason, current: Resolution, yes: u32, no: u32, t: &Thresholds) -> Resolution {
    let next = step(reason, current, yes, no, t);
    if next == current || !next.active() {
        return next;
    }
    if next == Resolution::UnderReview {
        match step(reason, Resolution::UnderReview, yes, no, t) {
            Resolution::Hidden => return Resolution::Hidden,
            Resolution::Removed if t.tiny => return Resolution::Removed,
            _ => {}
        }
    }
    next
}

/// web's `next_resolution`.
fn step(reason: Reason, current: Resolution, yes: u32, no: u32, t: &Thresholds) -> Resolution {
    let reviewing = matches!(current, Resolution::UnderReview | Resolution::Hidden);
    if reviewing && !t.can_still_delete(yes, no) {
        return Resolution::Dismissed;
    }
    if current == Resolution::Open && yes >= 1 {
        if t.tiny && t.delete_ready(yes, no) {
            return Resolution::Removed;
        }
        return Resolution::UnderReview;
    }
    if reviewing && t.delete_ready(yes, no) {
        return Resolution::Removed;
    }
    if current == Resolution::UnderReview && t.hide_ready(reason, yes, no) {
        return Resolution::Hidden;
    }
    if current.active() {
        current
    } else {
        Resolution::Open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(at: i64, voter: u8, yes: bool) -> Ballot<u8> {
        Ballot { at, voter, yes }
    }

    #[test]
    fn spam_needs_three_yes_votes_in_a_small_audience() {
        let one = [b(1, 1, true)];
        assert_eq!(
            replay(Reason::Spam, 3, 0, 0, &one).resolution,
            Resolution::UnderReview
        );
        let three = [b(1, 1, true), b(2, 2, true), b(3, 3, true)];
        let out = replay(Reason::Spam, 3, 0, 0, &three);
        assert_eq!(out.resolution, Resolution::Removed);
        assert_eq!((out.yes, out.no), (3, 0));
    }

    #[test]
    fn no_votes_dismiss_once_deletion_is_out_of_reach() {
        let votes = [b(1, 1, true), b(2, 2, false), b(3, 3, false)];
        assert_eq!(
            replay(Reason::Spam, 3, 0, 0, &votes).resolution,
            Resolution::Dismissed
        );
    }

    #[test]
    fn a_report_without_counted_yes_stays_open() {
        assert_eq!(
            replay::<u8>(Reason::Spam, 3, 0, 0, &[]).resolution,
            Resolution::Open
        );
        let no = [b(1, 2, false)];
        assert_eq!(
            replay(Reason::Spam, 3, 0, 0, &no).resolution,
            Resolution::Open
        );
    }

    #[test]
    fn serious_harm_hides_before_it_deletes() {
        let votes = [b(1, 1, true), b(2, 2, true), b(3, 3, true)];
        let out = replay(Reason::SeriousHarm, 5, 0, 0, &votes);
        assert_eq!(out.resolution, Resolution::Hidden);
        let more = [&votes[..], &[b(4, 4, true), b(5, 5, true)]].concat();
        assert_eq!(
            replay(Reason::SeriousHarm, 5, 0, 0, &more).resolution,
            Resolution::Removed
        );
    }

    #[test]
    fn a_changed_vote_replaces_the_old_one_and_order_is_by_time() {
        let votes = [b(3, 2, false), b(1, 1, true), b(2, 2, true)];
        let out = replay(Reason::Spam, 3, 0, 0, &votes);
        assert_eq!((out.yes, out.no), (1, 1));
        assert_eq!(out.resolution, Resolution::UnderReview);
    }

    #[test]
    fn votes_after_the_end_change_nothing() {
        let votes = [b(1, 1, true), b(2, 2, true), b(3, 3, true), b(4, 3, false)];
        let out = replay(Reason::Spam, 3, 0, 0, &votes);
        assert_eq!(out.resolution, Resolution::Removed);
        assert_eq!((out.yes, out.no), (3, 0));
    }

    #[test]
    fn one_yes_removes_a_message_in_a_one_to_one_chat() {
        let out = replay_chat(Reason::Spam, 1, 0, &[b(1, 1, true)]);
        assert_eq!(out.resolution, Resolution::Removed);
        let harm = replay_chat(Reason::SeriousHarm, 1, 0, &[b(1, 1, true)]);
        assert_eq!(harm.resolution, Resolution::Removed);
    }

    #[test]
    fn a_small_group_reviews_and_then_dismisses() {
        // Three people: two others can't reach web's 3-vote floor.
        let one = [b(1, 1, true)];
        assert_eq!(
            replay_chat(Reason::Spam, 2, 0, &one).resolution,
            Resolution::UnderReview
        );
        let two = [b(1, 1, true), b(2, 2, true)];
        assert_eq!(
            replay_chat(Reason::Spam, 2, 0, &two).resolution,
            Resolution::Dismissed
        );
        let four = [b(1, 1, true), b(2, 2, true), b(3, 3, true)];
        assert_eq!(
            replay_chat(Reason::Spam, 3, 0, &four).resolution,
            Resolution::Removed
        );
    }

    #[test]
    fn older_items_need_a_larger_share() {
        // Two yes, one no is 66.7%: enough when new, not at a week old (86%).
        let week = 7 * DAY_MICROS;
        let votes = [
            b(week, 1, true),
            b(week + 1, 2, false),
            b(week + 2, 3, true),
        ];
        assert_eq!(
            replay(Reason::Spam, 3, 0, 0, &votes).resolution,
            Resolution::Dismissed
        );
        let fresh = [b(1, 1, true), b(2, 2, false), b(3, 3, true)];
        assert_eq!(
            replay(Reason::Spam, 3, 0, 0, &fresh).resolution,
            Resolution::Removed
        );
    }
}
