use std::collections::{BTreeMap, BTreeSet};

use crate::tally::{tally, Tally};
use crate::{APPEAL_AFTER_US, COOLDOWN_US, MARKS_TO_OPEN, REVIEW_US};

/// What one link says. `Mark(true)` marks the account as a bot, `Mark(false)` clears the
/// mark; `Vote(true)` votes bot, `Vote(false)` real person; `Appeal` is the account asking
/// for its re-review.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Mark(bool),
    Vote(bool),
    Appeal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Act<A> {
    pub who: A,
    pub at: i64,
    pub kind: Kind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenReview<A> {
    pub opened_at: i64,
    pub is_appeal: bool,
    /// Each counted voter's latest vote (true = bot).
    pub votes: BTreeMap<A, bool>,
}

impl<A> OpenReview<A> {
    #[must_use]
    pub fn closes_at(&self) -> i64 {
        self.opened_at + REVIEW_US
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Closed {
    pub at: i64,
    pub upheld: bool,
    pub is_appeal: bool,
}

/// A review opening or closing, in order (web's notifications follow these).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Logged {
    Opened { at: i64, is_appeal: bool },
    Closed(Closed),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State<A> {
    pub log: Vec<Logged>,
    pub open: Option<OpenReview<A>>,
    pub last_closed: Option<Closed>,
    /// Live bot marks (shown on the profile's Trust sheet), and those that count.
    pub markers: BTreeSet<A>,
    pub counted_markers: BTreeSet<A>,
    /// Markers of a cleared review, and until when they can't mark again.
    pub cooling: BTreeMap<A, i64>,
    /// When the account was paused: from, and until (an appeal that cleared it).
    pub paused: Vec<(i64, Option<i64>)>,
}

impl<A: Ord + Clone> State<A> {
    fn new() -> Self {
        Self {
            log: Vec::new(),
            open: None,
            last_closed: None,
            markers: BTreeSet::new(),
            counted_markers: BTreeSet::new(),
            cooling: BTreeMap::new(),
            paused: Vec::new(),
        }
    }

    #[must_use]
    pub fn paused_at(&self, at: i64) -> bool {
        self.paused
            .iter()
            .any(|(from, to)| *from <= at && to.is_none_or(|t| at < t))
    }

    #[must_use]
    pub fn cooling_at(&self, who: &A, at: i64) -> bool {
        self.cooling.get(who).is_some_and(|until| at < *until)
    }

    #[must_use]
    pub fn tally(&self) -> Option<Tally> {
        self.open
            .as_ref()
            .map(|r| tally(&r.votes, &self.counted_markers))
    }

    /// When the account may ask for its re-review, if it still has one.
    #[must_use]
    pub fn re_review_from(&self) -> Option<i64> {
        match (self.open.as_ref(), self.last_closed) {
            (None, Some(c)) if c.upheld && !c.is_appeal => Some(c.at + APPEAL_AFTER_US),
            _ => None,
        }
    }

    fn close(&mut self, at: i64, upheld: bool) {
        let Some(review) = self.open.take() else {
            return;
        };
        let closed = Closed {
            at,
            upheld,
            is_appeal: review.is_appeal,
        };
        self.last_closed = Some(closed);
        self.log.push(Logged::Closed(closed));
        if upheld {
            if !review.is_appeal {
                self.paused.push((at, None));
            }
            return;
        }
        if let Some(last) = self.paused.last_mut() {
            if last.1.is_none() {
                last.1 = Some(at);
            }
        }
        for marker in std::mem::take(&mut self.markers) {
            self.cooling.insert(marker, at + COOLDOWN_US);
        }
        self.counted_markers.clear();
    }

    fn expire(&mut self, now: i64) {
        if let Some(closes) = self.open.as_ref().map(OpenReview::closes_at) {
            if now >= closes {
                self.close(closes, false);
            }
        }
    }

    fn settle(&mut self, at: i64) {
        let Some(t) = self.tally() else { return };
        if t.passing {
            self.close(at, true);
        } else if t.real_votes > 0 && !t.can_still_pass {
            self.close(at, false);
        }
    }

    fn open_review(&mut self, at: i64, is_appeal: bool) {
        self.log.push(Logged::Opened { at, is_appeal });
        self.open = Some(OpenReview {
            opened_at: at,
            is_appeal,
            votes: BTreeMap::new(),
        });
    }

    fn mark(&mut self, who: A, at: i64, bot: bool, counts: bool) {
        if !bot {
            self.markers.remove(&who);
            self.counted_markers.remove(&who);
            return;
        }
        if self.cooling_at(&who, at) {
            return;
        }
        self.markers.insert(who.clone());
        if counts {
            self.counted_markers.insert(who);
        }
        let opens = self.open.is_none()
            && !self.paused_at(at)
            && self.counted_markers.len() >= MARKS_TO_OPEN;
        if opens {
            self.open_review(at, false);
        }
    }

    fn appeal(&mut self, at: i64) {
        if self.re_review_from().is_some_and(|from| at >= from) {
            self.open_review(at, true);
        }
    }
}

/// Replay `acts` (any order) up to `now`. `counts(who, at)`: whether `who` had Full standing
/// at `at`; marks and votes from anyone else are kept for display but never count.
pub fn replay<A: Ord + Clone>(
    mut acts: Vec<Act<A>>,
    counts: impl Fn(&A, i64) -> bool,
    now: i64,
) -> State<A> {
    acts.sort_by_key(|a| a.at);
    let mut state = State::new();
    for act in acts.into_iter().filter(|a| a.at <= now) {
        state.expire(act.at);
        match act.kind {
            Kind::Mark(bot) => {
                let ok = bot && counts(&act.who, act.at);
                state.mark(act.who, act.at, bot, ok);
            }
            Kind::Vote(bot) => {
                if counts(&act.who, act.at) {
                    if let Some(review) = state.open.as_mut() {
                        review.votes.insert(act.who, bot);
                        state.settle(act.at);
                    }
                }
            }
            Kind::Appeal => state.appeal(act.at),
        }
    }
    state.expire(now);
    state
}
