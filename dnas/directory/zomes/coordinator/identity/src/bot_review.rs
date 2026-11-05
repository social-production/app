//! Public bot reviews (ADR 0074), web-backend's `bot_reviews.py`: bot marks, review votes and
//! the one re-review, as links on the account (identity integrity `bot_review`). The state is
//! replayed from them by every reader (`sp_bot_review::replay`).

use std::collections::BTreeMap;

use hdk::prelude::*;
use identity_integrity::{LinkTypes, MARK_BOT, VOTE_BOT};
use sp_bot_review::{replay, Act, Kind, State, COOLDOWN_US};

use crate::guest_error;
use crate::records::links;
use crate::standing::{full_since, own_proof};

#[derive(Serialize, Deserialize, Debug)]
pub struct BotReviewView {
    /// "none", "open" or "upheld" (the account is paused).
    pub status: String,
    pub is_appeal: bool,
    pub bot_votes: u32,
    pub real_votes: u32,
    pub votes_required: u32,
    pub closes_at: Option<Timestamp>,
    /// The caller's counted vote in the open review (true = bot).
    pub my_vote: Option<bool>,
    /// When the paused account may ask for its one re-review.
    pub re_review_from: Option<Timestamp>,
    /// Everyone whose bot mark is live (web's Trust sheet "Bot marks").
    pub markers: Vec<AgentPubKey>,
    /// Until when the caller can't mark this account again (a cleared review's marker).
    pub my_cooldown: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BotChoice {
    pub account: AgentPubKey,
    pub bot: bool,
}

fn now() -> ExternResult<i64> {
    Ok(sys_time()?.as_micros())
}

fn acts_of(link_type: LinkTypes, account: &AgentPubKey) -> ExternResult<Vec<Act<AgentPubKey>>> {
    Ok(links(account.clone(), link_type)?
        .into_iter()
        .map(|l| {
            let bot = l.tag.0.first() == Some(&MARK_BOT);
            let kind = match link_type {
                LinkTypes::BotMarks => Kind::Mark(bot),
                LinkTypes::BotReviewVotes => Kind::Vote(l.tag.0.first() == Some(&VOTE_BOT)),
                _ => Kind::Appeal,
            };
            Act {
                who: l.author,
                at: l.timestamp.as_micros(),
                kind,
            }
        })
        .collect())
}

/// The account's review, replayed up to now. `None` when nobody ever marked it (one link
/// read: no marks means no review, so nothing else can be on the account).
pub fn state(account: &AgentPubKey) -> ExternResult<Option<State<AgentPubKey>>> {
    let mut acts = acts_of(LinkTypes::BotMarks, account)?;
    if acts.is_empty() {
        return Ok(None);
    }
    acts.extend(acts_of(LinkTypes::BotReviewVotes, account)?);
    acts.extend(acts_of(LinkTypes::ReReviews, account)?);
    let mut since: BTreeMap<AgentPubKey, Option<i64>> = BTreeMap::new();
    for act in &acts {
        if !since.contains_key(&act.who) {
            since.insert(act.who.clone(), full_since(&act.who)?);
        }
    }
    let counts = |who: &AgentPubKey, at: i64| {
        since
            .get(who)
            .copied()
            .flatten()
            .is_some_and(|from| from < at)
    };
    Ok(Some(replay(acts, counts, now()?)))
}

/// Whether a review had found `agent` to be a bot at `at`.
pub fn paused_at(agent: &AgentPubKey, at: i64) -> ExternResult<bool> {
    Ok(state(agent)?.is_some_and(|s| s.paused_at(at)))
}

pub fn view(account: &AgentPubKey) -> ExternResult<BotReviewView> {
    let me = agent_info()?.agent_initial_pubkey;
    let s = state(account)?;
    let tally = s
        .as_ref()
        .and_then(State::tally)
        .unwrap_or(sp_bot_review::Tally {
            bot_votes: 0,
            real_votes: 0,
            votes_required: sp_bot_review::MIN_VOTES,
            passing: false,
            can_still_pass: false,
        });
    let Some(s) = s else {
        return Ok(BotReviewView {
            status: "none".into(),
            is_appeal: false,
            bot_votes: 0,
            real_votes: 0,
            votes_required: tally.votes_required,
            closes_at: None,
            my_vote: None,
            re_review_from: None,
            markers: Vec::new(),
            my_cooldown: None,
        });
    };
    let at = now()?;
    let upheld = s.paused_at(at);
    let status = match (&s.open, upheld) {
        (Some(_), _) => "open",
        (None, true) => "upheld",
        _ => "none",
    };
    let is_appeal = match &s.open {
        Some(r) => r.is_appeal,
        None => s.last_closed.is_some_and(|c| c.is_appeal),
    };
    Ok(BotReviewView {
        status: status.into(),
        is_appeal,
        bot_votes: tally.bot_votes,
        real_votes: tally.real_votes,
        votes_required: tally.votes_required,
        closes_at: s.open.as_ref().map(|r| Timestamp(r.closes_at())),
        my_vote: s.open.as_ref().and_then(|r| r.votes.get(&me).copied()),
        re_review_from: s.re_review_from().map(Timestamp),
        markers: s.markers.iter().cloned().collect(),
        my_cooldown: s
            .cooling
            .get(&me)
            .copied()
            .filter(|until| at < *until)
            .map(Timestamp),
    })
}

/// A full member who isn't paused (web's `ensure_not_paused` plus the FULL checks).
fn ensure_full_and_active(what: &str) -> ExternResult<AgentPubKey> {
    let me = agent_info()?.agent_initial_pubkey;
    if own_proof()?.is_none() {
        return Err(guest_error(format!(
            "{what} opens once your own votes count fully."
        )));
    }
    if paused_at(&me, now()?)? {
        return Err(guest_error(
            "A community review found this account is a bot, so it is read-only.",
        ));
    }
    Ok(me)
}

/// Mark `account` as a possible bot, or clear your mark. Marking replaces your vouch, as
/// web keeps one stance per pair.
pub fn set_mark(choice: BotChoice) -> ExternResult<ActionHash> {
    let me = agent_info()?.agent_initial_pubkey;
    if choice.account == me {
        return Err(guest_error("You can't mark yourself."));
    }
    if choice.bot {
        ensure_full_and_active("Marking as a bot")?;
        let cooling = state(&choice.account)?.and_then(|s| s.cooling.get(&me).copied());
        if cooling.is_some_and(|until| now().is_ok_and(|n| n < until)) {
            return Err(guest_error(
                "A review cleared this account, so you can't mark it again for 30 days.",
            ));
        }
        crate::standing::withdraw_my_vouch(&choice.account)?;
    }
    let tag = if choice.bot {
        MARK_BOT
    } else {
        identity_integrity::MARK_CLEAR
    };
    create_link(choice.account, me, LinkTypes::BotMarks, LinkTag(vec![tag]))
}

/// Whether the caller's live bot mark is on `account` (vouching clears it).
pub fn my_mark_is_live(account: &AgentPubKey) -> ExternResult<bool> {
    let me = agent_info()?.agent_initial_pubkey;
    Ok(state(account)?.is_some_and(|s| s.markers.contains(&me)))
}

pub fn vote(choice: BotChoice) -> ExternResult<ActionHash> {
    let me = ensure_full_and_active("Voting in bot reviews")?;
    if choice.account == me {
        return Err(guest_error("You can't vote on your own review"));
    }
    if state(&choice.account)?.and_then(|s| s.open).is_none() {
        return Err(guest_error("No open review"));
    }
    let tag = if choice.bot {
        VOTE_BOT
    } else {
        identity_integrity::VOTE_REAL
    };
    create_link(
        choice.account,
        me,
        LinkTypes::BotReviewVotes,
        LinkTag(vec![tag]),
    )
}

pub fn request_re_review() -> ExternResult<ActionHash> {
    let me = agent_info()?.agent_initial_pubkey;
    let Some(s) = state(&me)? else {
        return Err(guest_error("This account isn't paused"));
    };
    if s.open.is_some() {
        return Err(guest_error("A review is already open"));
    }
    if !s.paused_at(now()?) {
        return Err(guest_error("This account isn't paused"));
    }
    match s.re_review_from() {
        None => Err(guest_error("The one re-review has been used")),
        Some(from) if now()? < from => {
            Err(guest_error("A re-review opens 30 days after the review"))
        }
        Some(_) => create_link(me.clone(), me, LinkTypes::ReReviews, ()),
    }
}

/// Vouching pauses for 30 days after someone you vouched for is found to be a bot.
pub fn vouch_banned(vouchees: &[AgentPubKey]) -> ExternResult<bool> {
    let at = now()?;
    for vouchee in vouchees {
        if let Some(s) = state(vouchee)? {
            if s.paused.iter().any(|(from, _)| at - from < COOLDOWN_US) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
