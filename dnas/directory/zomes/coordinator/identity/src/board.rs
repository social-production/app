//! Platform moderators (ADR 0095), web-backend's `services/board.py`: candidacies on the
//! "board" path and votes on each account, replayed by every reader (`sp_board::replay`)
//! with the quorum from the weekly active count (`active.rs`).

use std::collections::{BTreeMap, BTreeSet};

use hdk::prelude::*;
use identity_integrity::{
    LinkTypes, BOARD_NEUTRAL, BOARD_NO, BOARD_PATH, BOARD_YES, VOLUNTEER, WITHDRAW,
};
use sp_active::counted_week;
use sp_board::{replay, Act, Kind, Standing, State};
use sp_governance::quorum::required_votes;

use crate::active::Counter;
use crate::guest_error;
use crate::records::links;
use crate::standing::full_before;

#[derive(Serialize, Deserialize, Debug)]
pub struct BoardRow {
    pub agent: AgentPubKey,
    /// "candidate", "moderator" or "grace" (a moderator short of the quorum).
    pub standing: String,
    pub grace_ends: Option<Timestamp>,
    pub yes: u32,
    pub no: u32,
    /// The caller's counted vote (true = yes).
    pub my_vote: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoardView {
    pub weekly_active: u32,
    pub quorum: u32,
    /// Moderators first, then candidates, most yes votes first.
    pub rows: Vec<BoardRow>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoardVote {
    pub account: AgentPubKey,
    /// "yes", "no" or "neutral" (withdraws the vote), as web.
    pub vote: String,
}

fn board_path() -> ExternResult<EntryHash> {
    Path::from(BOARD_PATH).path_entry_hash()
}

fn now() -> ExternResult<i64> {
    Ok(sys_time()?.as_micros())
}

/// Every Full member's act at the time they acted, by account (web's `ensure_established_voter`).
fn counted(
    acts: Vec<Link>,
    kind: impl Fn(u8) -> Option<Kind>,
) -> ExternResult<Vec<Act<AgentPubKey>>> {
    let mut out = Vec::new();
    for l in acts {
        let Some(k) = l.tag.0.first().copied().and_then(&kind) else {
            continue;
        };
        if l.target.into_agent_pub_key().as_ref() != Some(&l.author) {
            continue;
        }
        if full_before(&l.author, l.timestamp)? {
            out.push(Act {
                who: l.author,
                at: l.timestamp.as_micros(),
                kind: k,
            });
        }
    }
    Ok(out)
}

fn candidacy(tag: u8) -> Option<Kind> {
    match tag {
        VOLUNTEER => Some(Kind::Volunteer(true)),
        WITHDRAW => Some(Kind::Volunteer(false)),
        _ => None,
    }
}

fn vote_kind(tag: u8) -> Option<Kind> {
    match tag {
        BOARD_YES => Some(Kind::Vote(Some(true))),
        BOARD_NO => Some(Kind::Vote(Some(false))),
        BOARD_NEUTRAL => Some(Kind::Vote(None)),
        _ => None,
    }
}

/// Each account that ever volunteered, replayed to `at`, and the weekly active count at `at`.
pub fn states(at: i64) -> ExternResult<(BTreeMap<AgentPubKey, State<AgentPubKey>>, u32)> {
    let candidacies = counted(links(board_path()?, LinkTypes::BoardCandidacy)?, candidacy)?;
    let mut counter = Counter::default();
    let accounts: BTreeSet<AgentPubKey> = candidacies.iter().map(|a| a.who.clone()).collect();
    let mut acts: BTreeMap<AgentPubKey, Vec<Act<AgentPubKey>>> = BTreeMap::new();
    for c in candidacies {
        acts.entry(c.who.clone()).or_default().push(c);
    }
    for account in &accounts {
        let votes = counted(links(account.clone(), LinkTypes::BoardVotes)?, vote_kind)?;
        acts.entry(account.clone()).or_default().extend(votes);
    }
    // The quorum at any time: `required_votes` of the last complete week's count.
    let first = acts.values().flatten().map(|a| a.at).min().unwrap_or(at);
    let mut quorums = BTreeMap::new();
    for week in counted_week(first)..=counted_week(at) {
        quorums.insert(week, required_votes(counter.week(week)?));
    }
    let quorum = |t: i64| quorums.get(&counted_week(t)).copied().unwrap_or(0);
    let out = acts
        .iter()
        .map(|(who, acts)| (who.clone(), replay(who, acts, at, quorum)))
        .collect();
    let weekly = counter.week(counted_week(at))?;
    Ok((out, weekly))
}

pub fn view() -> ExternResult<BoardView> {
    let me = agent_info()?.agent_initial_pubkey;
    let at = now()?;
    let (states, weekly_active) = states(at)?;
    let mut rows: Vec<BoardRow> = states
        .into_iter()
        .filter_map(|(agent, s)| {
            let (standing, grace_ends) = match s.standing {
                Standing::None => return None,
                Standing::Candidate => ("candidate", None),
                Standing::Moderator => ("moderator", None),
                Standing::Grace { ends } => ("grace", Some(Timestamp(ends))),
            };
            Some(BoardRow {
                my_vote: s.votes.get(&me).copied(),
                agent,
                standing: standing.into(),
                grace_ends,
                yes: s.stats.yes,
                no: s.stats.no,
            })
        })
        .collect();
    rows.sort_by_key(|r| (r.standing == "candidate", std::cmp::Reverse(r.yes)));
    Ok(BoardView {
        weekly_active,
        quorum: required_votes(weekly_active),
        rows,
    })
}

/// Whether `agent` moderates now (a moderator in grace still does), for merge capability.
pub fn is_moderator(agent: &AgentPubKey) -> ExternResult<bool> {
    let (states, _) = states(now()?)?;
    Ok(states
        .get(agent)
        .is_some_and(|s| matches!(s.standing, Standing::Moderator | Standing::Grace { .. })))
}

fn ensure_full(what: &str) -> ExternResult<AgentPubKey> {
    let me = agent_info()?.agent_initial_pubkey;
    if !full_before(&me, sys_time()?)? {
        return Err(guest_error(format!(
            "{what} opens once your own votes count fully."
        )));
    }
    Ok(me)
}

pub fn volunteer(on: bool) -> ExternResult<ActionHash> {
    let me = ensure_full("Volunteering as a moderator")?;
    let tag = if on { VOLUNTEER } else { WITHDRAW };
    let hash = create_link(
        board_path()?,
        me,
        LinkTypes::BoardCandidacy,
        LinkTag(vec![tag]),
    )?;
    crate::active::record()?;
    Ok(hash)
}

pub fn vote(input: BoardVote) -> ExternResult<ActionHash> {
    let me = ensure_full("Voting on moderators")?;
    if input.account == me {
        return Err(guest_error("You can't vote on yourself."));
    }
    let tag = match input.vote.as_str() {
        "yes" => BOARD_YES,
        "no" => BOARD_NO,
        "neutral" => BOARD_NEUTRAL,
        _ => return Err(guest_error("vote must be one of: ['no', 'yes']")),
    };
    let (states, _) = states(now()?)?;
    let listed = states
        .get(&input.account)
        .is_some_and(|s| s.standing != Standing::None);
    if !listed && tag != BOARD_NEUTRAL {
        return Err(guest_error("Board candidate/member not found"));
    }
    let hash = create_link(input.account, me, LinkTypes::BoardVotes, LinkTag(vec![tag]))?;
    crate::active::record()?;
    Ok(hash)
}
