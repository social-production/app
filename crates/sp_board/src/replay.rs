use std::collections::BTreeMap;

use crate::{week_start, APPROVAL_PERCENT, GRACE_US, WEEK_US};

/// What one link says. `Volunteer(true)` volunteers, `Volunteer(false)` withdraws (web's
/// `remove_volunteer`); `Vote(Some(true))` votes yes, `Some(false)` no, `None` withdraws the
/// vote (web's "neutral").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Volunteer(bool),
    Vote(Option<bool>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Act<A> {
    /// The person acting: the account itself for a candidacy, the voter for a vote.
    pub who: A,
    pub at: i64,
    pub kind: Kind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Standing {
    /// Not volunteering (never did, withdrew, or was removed).
    None,
    Candidate,
    Moderator,
    /// A moderator short of the quorum, until `ends`.
    Grace {
        ends: i64,
    },
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub yes: u32,
    pub no: u32,
}

impl Stats {
    #[must_use]
    pub fn votes(self) -> u32 {
        self.yes + self.no
    }

    /// web's `approval_ratio >= MIN_APPROVAL_RATIO`, in integers.
    #[must_use]
    pub fn approved(self) -> bool {
        self.yes * 100 >= APPROVAL_PERCENT * self.votes()
    }

    /// web's `_meets_standing_threshold`: enough votes, and (when there are any) 66% yes.
    #[must_use]
    pub fn qualifies(self, quorum: u32) -> bool {
        self.votes() >= quorum && (self.votes() == 0 || self.approved())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State<A> {
    pub standing: Standing,
    pub stats: Stats,
    /// Each counted voter's latest yes (true) or no (false).
    pub votes: BTreeMap<A, bool>,
    /// When the account last became a moderator, and when it last stopped being one.
    pub seated_at: Option<i64>,
    pub removed_at: Option<i64>,
}

/// The account `me`'s standing at `now`, from its candidacy acts and the votes on it, in any
/// order. The caller passes only acts that count: candidacies by `me`, and votes by people
/// with Full standing when they voted. `quorum(at)` is the votes needed at `at`.
///
/// web reconciles standings whenever anyone volunteers or votes. Here they are checked at each
/// act on this account and at the start of each week (when the weekly active count, and so
/// the quorum, changes), so every reader agrees whenever it reads. A grace period starts at
/// the first check that finds a moderator short of the quorum; removal takes effect when it
/// ends (seen at `now`, or at the next check).
pub fn replay<A: Ord + Clone>(
    me: &A,
    acts: &[Act<A>],
    now: i64,
    quorum: impl Fn(i64) -> u32,
) -> State<A> {
    let mut sorted: Vec<&Act<A>> = acts.iter().filter(|a| a.at <= now).collect();
    sorted.sort_by_key(|a| a.at);
    let mut state = State {
        standing: Standing::None,
        stats: Stats::default(),
        votes: BTreeMap::new(),
        seated_at: None,
        removed_at: None,
    };
    let mut boundary = sorted.first().map(|a| week_start(a.at) + WEEK_US);
    for act in sorted {
        while let Some(b) = boundary.filter(|b| *b <= act.at) {
            check(&mut state, b, quorum(b));
            boundary = Some(b + WEEK_US);
        }
        match act.kind {
            Kind::Volunteer(on) if act.who == *me => {
                check(&mut state, act.at, quorum(act.at));
                state.standing = match (on, state.standing) {
                    (false, Standing::Moderator | Standing::Grace { .. }) => {
                        state.removed_at = Some(act.at);
                        Standing::None
                    }
                    (false, _) => Standing::None,
                    // web: volunteering again as a moderator changes nothing.
                    (true, Standing::Moderator | Standing::Grace { .. }) => state.standing,
                    (true, _) => Standing::Candidate,
                };
            }
            Kind::Vote(vote) if act.who != *me => {
                match vote {
                    Some(yes) => state.votes.insert(act.who.clone(), yes),
                    None => state.votes.remove(&act.who),
                };
                state.stats = stats(&state.votes);
            }
            _ => continue,
        }
        check(&mut state, act.at, quorum(act.at));
    }
    while let Some(b) = boundary.filter(|b| *b <= now) {
        check(&mut state, b, quorum(b));
        boundary = Some(b + WEEK_US);
    }
    if let Standing::Grace { ends } = state.standing {
        if now > ends {
            state.standing = Standing::None;
            state.removed_at = Some(ends);
        }
    }
    state
}

fn stats<A>(votes: &BTreeMap<A, bool>) -> Stats {
    let yes = u32::try_from(votes.values().filter(|v| **v).count()).unwrap_or(u32::MAX);
    let total = u32::try_from(votes.len()).unwrap_or(u32::MAX);
    Stats {
        yes,
        no: total - yes,
    }
}

/// web's `_promote_qualified_candidates` then `_remove_unqualified_members`, for one account.
fn check<A>(state: &mut State<A>, at: i64, quorum: u32) {
    let s = state.stats;
    if state.standing == Standing::Candidate && s.qualifies(quorum) {
        state.standing = Standing::Moderator;
        state.seated_at = Some(at);
    }
    let remove = |state: &mut State<A>, when: i64| {
        state.standing = Standing::None;
        state.removed_at = Some(when);
    };
    match state.standing {
        Standing::Moderator | Standing::Grace { .. } if s.votes() > 0 && !s.approved() => {
            remove(state, at);
        }
        Standing::Moderator | Standing::Grace { .. } if s.votes() >= quorum => {
            state.standing = Standing::Moderator;
        }
        Standing::Moderator => {
            state.standing = Standing::Grace {
                ends: at + GRACE_US,
            }
        }
        Standing::Grace { ends } if at > ends => remove(state, ends),
        _ => {}
    }
}
