//! Project DNA API (docs/plans/M2-projects.md). Reads merge the network's view with the
//! caller's own chain, so your own join/leave/signal shows at once.

mod charter;
mod own;

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::{creator, Charter, LinkTypes, DEMAND, OPPOSE};

pub(crate) const CHARTER: &str = "charter";
pub(crate) const MEMBERS: &str = "members";
pub(crate) const SIGNALS: &str = "signals";

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stance {
    Demand,
    Oppose,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectView {
    pub creator: AgentPubKey,
    pub charter: Option<Charter>,
    pub members: Vec<AgentPubKey>,
    pub demand: u32,
    pub opposition: u32,
    pub am_member: bool,
    pub my_signal: Option<Stance>,
    pub created_at: Option<Timestamp>,
    pub last_activity_at: Option<Timestamp>,
}

#[hdk_extern]
pub fn create_charter(charter: Charter) -> ExternResult<ActionHash> {
    charter::create(charter)
}

#[hdk_extern]
pub fn update_charter(charter: Charter) -> ExternResult<ActionHash> {
    charter::update(charter)
}

#[hdk_extern]
pub fn join() -> ExternResult<()> {
    if own::live_links(LinkTypes::Members)?.is_empty() {
        let me = agent_info()?.agent_initial_pubkey;
        create_link(anchor(MEMBERS)?, me, LinkTypes::Members, ())?;
    }
    Ok(())
}

#[hdk_extern]
pub fn leave() -> ExternResult<()> {
    for (hash, _) in own::live_links(LinkTypes::Members)? {
        delete_link(hash, GetOptions::default())?;
    }
    Ok(())
}

/// One live signal per agent: replaces any earlier one; `None` withdraws it.
#[hdk_extern]
pub fn set_signal(stance: Option<Stance>) -> ExternResult<()> {
    for (hash, _) in own::live_links(LinkTypes::Signals)? {
        delete_link(hash, GetOptions::default())?;
    }
    if let Some(stance) = stance {
        let me = agent_info()?.agent_initial_pubkey;
        let tag = match stance {
            Stance::Demand => DEMAND,
            Stance::Oppose => OPPOSE,
        };
        create_link(
            anchor(SIGNALS)?,
            me,
            LinkTypes::Signals,
            LinkTag::new([tag]),
        )?;
    }
    Ok(())
}

#[hdk_extern]
pub fn get_project() -> ExternResult<Option<ProjectView>> {
    let Some(creator) = creator()? else {
        return Ok(None);
    };
    let me = agent_info()?.agent_initial_pubkey;
    let (charter, charter_times) = charter::current()?;
    let member_links = links(MEMBERS, LinkTypes::Members)?;
    let signal_links = links(SIGNALS, LinkTypes::Signals)?;

    let mut members: Vec<AgentPubKey> = member_links.iter().map(|l| l.author.clone()).collect();
    let am_member = !own::live_links(LinkTypes::Members)?.is_empty();
    members.retain(|a| a != &me);
    members.sort();
    members.dedup();
    if am_member {
        members.push(me.clone());
    }

    let mut stances: BTreeMap<AgentPubKey, (Timestamp, u8)> = BTreeMap::new();
    for l in &signal_links {
        let entry = stances
            .entry(l.author.clone())
            .or_insert((l.timestamp, l.tag.0[0]));
        if l.timestamp >= entry.0 {
            *entry = (l.timestamp, l.tag.0[0]);
        }
    }
    let my_signal = own::live_links(LinkTypes::Signals)?
        .first()
        .map(|(_, tag)| stance_of(*tag));
    stances.remove(&me);
    let count = |want: u8| stances.values().filter(|(_, t)| *t == want).count();
    let mine = |want: Stance| usize::from(my_signal == Some(want));

    let times = member_links
        .iter()
        .chain(&signal_links)
        .map(|l| l.timestamp);
    let last_activity_at = charter_times.iter().copied().chain(times).max();
    Ok(Some(ProjectView {
        creator,
        charter,
        members,
        demand: u32::try_from(count(DEMAND) + mine(Stance::Demand)).unwrap_or(u32::MAX),
        opposition: u32::try_from(count(OPPOSE) + mine(Stance::Oppose)).unwrap_or(u32::MAX),
        am_member,
        my_signal,
        created_at: charter_times.first().copied(),
        last_activity_at,
    }))
}

fn stance_of(tag: u8) -> Stance {
    if tag == OPPOSE {
        Stance::Oppose
    } else {
        Stance::Demand
    }
}

pub(crate) fn anchor(name: &str) -> ExternResult<EntryHash> {
    Path::from(name).path_entry_hash()
}

pub(crate) fn links(name: &str, link_type: LinkTypes) -> ExternResult<Vec<Link>> {
    get_links(
        LinkQuery::try_new(anchor(name)?, link_type)?,
        GetStrategy::default(),
    )
}
