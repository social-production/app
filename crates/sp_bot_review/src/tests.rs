use std::collections::{BTreeMap, BTreeSet};

use crate::*;

const H: i64 = 60 * 60 * 1_000_000;
const D: i64 = 24 * H;

fn mark(who: u8, at: i64) -> Act<u8> {
    Act {
        who,
        at,
        kind: Kind::Mark(true),
    }
}
fn vote(who: u8, at: i64, bot: bool) -> Act<u8> {
    Act {
        who,
        at,
        kind: Kind::Vote(bot),
    }
}
fn all_full(_: &u8, _: i64) -> bool {
    true
}

#[test]
fn three_counted_marks_open_a_review() {
    let s = replay(vec![mark(1, 1), mark(2, 2)], all_full, 10);
    assert!(s.open.is_none());
    let s = replay(vec![mark(1, 1), mark(2, 2), mark(3, 3)], all_full, 10);
    assert_eq!(s.open.as_ref().map(|r| r.opened_at), Some(3));
    assert_eq!(
        s.open.as_ref().map(OpenReview::closes_at),
        Some(3 + REVIEW_US)
    );
}

#[test]
fn marks_from_new_members_never_count() {
    let full = |who: &u8, _: i64| *who != 3;
    let s = replay(vec![mark(1, 1), mark(2, 2), mark(3, 3)], full, 10);
    assert!(s.open.is_none());
    assert_eq!(s.markers.len(), 3);
    assert_eq!(s.counted_markers.len(), 2);
}

#[test]
fn a_cleared_mark_stops_counting() {
    let clear = Act {
        who: 2,
        at: 2,
        kind: Kind::Mark(false),
    };
    let s = replay(
        vec![mark(1, 1), mark(2, 1), clear, mark(3, 3)],
        all_full,
        10,
    );
    assert!(s.open.is_none());
}

#[test]
fn three_bot_votes_uphold_and_pause() {
    let mut acts = vec![mark(1, 1), mark(2, 2), mark(3, 3)];
    acts.extend([vote(4, 4, true), vote(5, 5, true), vote(6, 6, true)]);
    let s = replay(acts.clone(), all_full, 10);
    // Six people involved: required_votes(6) is above three.
    assert!(s.open.is_some(), "{:?}", s.tally());
    acts.extend([vote(1, 7, true), vote(2, 8, true)]);
    let s = replay(acts, all_full, 10);
    assert!(s.open.is_none());
    assert_eq!(s.last_closed.map(|c| c.upheld), Some(true));
    assert!(s.paused_at(9));
    assert!(!s.paused_at(5));
    assert_eq!(
        s.re_review_from(),
        Some(s.last_closed.unwrap().at + APPEAL_AFTER_US)
    );
}

#[test]
fn real_votes_clear_early_and_start_the_cooldown() {
    let mut acts = vec![mark(1, 1), mark(2, 2), mark(3, 3)];
    acts.extend([vote(4, 4, false), vote(5, 5, false)]);
    let s = replay(acts, all_full, 10);
    assert_eq!(s.last_closed.map(|c| c.upheld), Some(false));
    assert!(s.markers.is_empty());
    assert!(s.cooling_at(&1, 6));
    assert!(!s.cooling_at(&1, 5 + COOLDOWN_US));
    // A cooling marker's new mark is ignored.
    let mut again = vec![mark(1, 1), mark(2, 2), mark(3, 3)];
    again.extend([vote(4, 4, false), vote(5, 5, false), mark(1, 6)]);
    assert!(replay(again, all_full, 10).markers.is_empty());
}

#[test]
fn an_undecided_review_clears_after_a_week() {
    let acts = vec![mark(1, 1), mark(2, 2), mark(3, 3), vote(4, 4, true)];
    let s = replay(acts.clone(), all_full, 3 + REVIEW_US - 1);
    assert!(s.open.is_some());
    let s = replay(acts, all_full, 3 + REVIEW_US);
    assert_eq!(
        s.last_closed,
        Some(Closed {
            at: 3 + REVIEW_US,
            upheld: false,
            is_appeal: false
        })
    );
    // A vote after the week changes nothing.
    let late = vec![
        mark(1, 1),
        mark(2, 2),
        mark(3, 3),
        vote(4, 3 + REVIEW_US + 1, true),
    ];
    assert!(replay(late, all_full, 4 * REVIEW_US).open.is_none());
}

fn upheld_at_9() -> Vec<Act<u8>> {
    let mut acts = vec![mark(1, 1), mark(2, 2), mark(3, 3)];
    acts.extend((1..=3).map(|w| vote(w, 3 + i64::from(w), true)));
    acts
}

#[test]
fn the_one_re_review_opens_after_thirty_days() {
    let s = replay(upheld_at_9(), all_full, 10);
    let closed = s.last_closed.unwrap().at;
    let early = Act {
        who: 0,
        at: closed + APPEAL_AFTER_US - 1,
        kind: Kind::Appeal,
    };
    let mut acts = upheld_at_9();
    acts.push(early);
    assert!(replay(acts, all_full, 60 * D).open.is_none());

    let appeal_at = closed + APPEAL_AFTER_US;
    let mut acts = upheld_at_9();
    acts.push(Act {
        who: 0,
        at: appeal_at,
        kind: Kind::Appeal,
    });
    let s = replay(acts.clone(), all_full, appeal_at + 1);
    assert!(s.open.as_ref().is_some_and(|r| r.is_appeal));
    assert!(
        s.paused_at(appeal_at + 1),
        "paused while the re-review runs"
    );

    acts.extend([vote(4, appeal_at + 2, false), vote(5, appeal_at + 3, false)]);
    let s = replay(acts.clone(), all_full, appeal_at + 10);
    assert_eq!(
        s.last_closed.map(|c| (c.upheld, c.is_appeal)),
        Some((false, true))
    );
    assert!(!s.paused_at(appeal_at + 10));
    assert!(s.paused_at(appeal_at));
}

#[test]
fn an_upheld_re_review_is_final() {
    let s = replay(upheld_at_9(), all_full, 10);
    let appeal_at = s.last_closed.unwrap().at + APPEAL_AFTER_US;
    let mut acts = upheld_at_9();
    acts.push(Act {
        who: 0,
        at: appeal_at,
        kind: Kind::Appeal,
    });
    acts.extend((1..=3).map(|w| vote(w, appeal_at + i64::from(w), true)));
    let s = replay(acts, all_full, appeal_at + 10);
    assert_eq!(
        s.last_closed.map(|c| (c.upheld, c.is_appeal)),
        Some((true, true))
    );
    assert!(s.paused_at(appeal_at + 10));
    assert_eq!(s.re_review_from(), None);
}

#[test]
fn tally_follows_web() {
    let markers: BTreeSet<u8> = [1, 2, 3].into();
    let votes: BTreeMap<u8, bool> = [(1, true), (2, true)].into();
    let t = tally(&votes, &markers);
    assert_eq!(
        (t.votes_required, t.passing, t.can_still_pass),
        (3, false, true)
    );
    let votes: BTreeMap<u8, bool> = [(1, true), (2, true), (3, false)].into();
    let t = tally(&votes, &markers);
    // 2 of 3 is 66.7%: passes.
    assert!(t.passing);
}

#[test]
fn the_log_lists_opens_and_closes() {
    let mut acts = vec![mark(1, 1), mark(2, 2), mark(3, 3)];
    acts.extend([vote(4, 4, false), vote(5, 5, false)]);
    let s = replay(acts, all_full, 10);
    assert_eq!(
        s.log,
        [
            Logged::Opened {
                at: 3,
                is_appeal: false
            },
            Logged::Closed(Closed {
                at: 5,
                upheld: false,
                is_appeal: false
            }),
        ]
    );
}
