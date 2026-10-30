//! Membership and signals: links from the space to their author (ADR 0030, 0065). Reads merge
//! the network's view with the caller's own chain, so your own join/leave/signal shows at once.

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::{LinkTypes, DEMAND, OPPOSE};

use crate::own::{live_links, live_links_at};
use crate::{links, Stance};

/// Join once; after a passed removal of you (ADR 0068), a membership made before it no longer
/// counts, so joining again replaces it with a new one.
pub fn join(removed: &[(AgentPubKey, Timestamp)]) -> ExternResult<()> {
    let me = agent_info()?.agent_initial_pubkey;
    let removed_at = removed
        .iter()
        .filter(|(agent, _)| agent == &me)
        .map(|(_, at)| *at)
        .max();
    let mine = live_links_at(LinkTypes::Members)?;
    let stale = |at: &Timestamp| removed_at.is_some_and(|r| *at <= r);
    if mine.iter().any(|(_, at)| !stale(at)) {
        return Ok(());
    }
    for (hash, _) in mine {
        delete_link(hash, GetOptions::default())?;
    }
    create_link(space_scope::base()?, me, LinkTypes::Members, ())?;
    Ok(())
}

pub fn leave() -> ExternResult<()> {
    for (hash, _) in live_links(LinkTypes::Members)? {
        delete_link(hash, GetOptions::default())?;
    }
    Ok(())
}

/// One live signal per agent: replaces any earlier one; `None` withdraws it.
pub fn set_signal(stance: Option<Stance>) -> ExternResult<()> {
    for (hash, _) in live_links(LinkTypes::Signals)? {
        delete_link(hash, GetOptions::default())?;
    }
    if let Some(stance) = stance {
        let me = agent_info()?.agent_initial_pubkey;
        let tag = match stance {
            Stance::Demand => DEMAND,
            Stance::Oppose => OPPOSE,
        };
        create_link(
            space_scope::base()?,
            me,
            LinkTypes::Signals,
            LinkTag::new([tag]),
        )?;
    }
    Ok(())
}

/// The caller's own live membership link.
pub fn my_membership() -> ExternResult<Option<ActionHash>> {
    Ok(live_links(LinkTypes::Members)?
        .into_iter()
        .next()
        .map(|(hash, _)| hash))
}

/// Each member's earliest live membership link: a proposal's population (ADR 0033).
pub fn snapshot() -> ExternResult<Vec<ActionHash>> {
    let me = agent_info()?.agent_initial_pubkey;
    let mut earliest: BTreeMap<AgentPubKey, (Timestamp, ActionHash)> = BTreeMap::new();
    for l in links(LinkTypes::Members)? {
        let entry = earliest
            .entry(l.author.clone())
            .or_insert((l.timestamp, l.create_link_hash.clone()));
        if l.timestamp < entry.0 {
            *entry = (l.timestamp, l.create_link_hash);
        }
    }
    earliest.remove(&me);
    let mut out: Vec<ActionHash> = earliest.into_values().map(|(_, hash)| hash).collect();
    out.extend(my_membership()?);
    Ok(out)
}

pub struct People {
    pub members: Vec<AgentPubKey>,
    pub am_member: bool,
    pub demand: u32,
    pub opposition: u32,
    pub my_signal: Option<Stance>,
    pub last_at: Option<Timestamp>,
}

pub fn read() -> ExternResult<People> {
    read_without(&[])
}

/// As `read`, leaving out memberships made before a passed removal of their member (a
/// channel's or community's `RemoveMember`, ADR 0068); joining again afterwards counts.
pub fn read_without(removed: &[(AgentPubKey, Timestamp)]) -> ExternResult<People> {
    let me = agent_info()?.agent_initial_pubkey;
    let gone = |l: &Link| {
        removed
            .iter()
            .any(|(agent, at)| agent == &l.author && l.timestamp <= *at)
    };
    let member_links: Vec<Link> = links(LinkTypes::Members)?
        .into_iter()
        .filter(|l| !gone(l))
        .collect();
    let signal_links = links(LinkTypes::Signals)?;

    // Your own join shows at once (your chain); after a removal only a later join counts.
    let removed_me = removed.iter().any(|(agent, _)| agent == &me);
    let am_member =
        my_membership()?.is_some() && (!removed_me || member_links.iter().any(|l| l.author == me));
    let mut members: Vec<AgentPubKey> = member_links
        .iter()
        .map(|l| l.author.clone())
        .filter(|a| a != &me)
        .collect();
    members.sort();
    members.dedup();
    if am_member {
        members.push(me.clone());
    }

    let mut stances: BTreeMap<AgentPubKey, (Timestamp, u8)> = BTreeMap::new();
    for l in &signal_links {
        let tag = l.tag.0.first().copied().unwrap_or(DEMAND);
        let entry = stances
            .entry(l.author.clone())
            .or_insert((l.timestamp, tag));
        if l.timestamp >= entry.0 {
            *entry = (l.timestamp, tag);
        }
    }
    stances.remove(&me);
    let my_signal = live_links(LinkTypes::Signals)?
        .first()
        .map(|(_, tag)| stance_of(*tag));
    let count = |want: u8, mine: Stance| {
        let others = stances.values().filter(|(_, t)| *t == want).count();
        u32::try_from(others + usize::from(my_signal == Some(mine))).unwrap_or(u32::MAX)
    };
    let last_at = member_links
        .iter()
        .chain(&signal_links)
        .map(|l| l.timestamp)
        .max();
    Ok(People {
        members,
        am_member,
        demand: count(DEMAND, Stance::Demand),
        opposition: count(OPPOSE, Stance::Oppose),
        my_signal,
        last_at,
    })
}

fn stance_of(tag: u8) -> Stance {
    if tag == OPPOSE {
        Stance::Oppose
    } else {
        Stance::Demand
    }
}
