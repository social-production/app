//! Platform feedback (ADR 0054). Items hang off a "feedback" anchor; votes are links from an
//! item to the voter, the newest per voter counting. Sorting (web's trending rule) is the UI's.

use std::collections::{BTreeMap, BTreeSet};

use hdk::prelude::*;
use identity_integrity::{EntryTypes, Feedback, FeedbackKind, LinkTypes, VOTE_DOWN, VOTE_UP};

use crate::records::{action_targets, decode, links, live};

const ALL_FEEDBACK: &str = "feedback";

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VoteDirection {
    Up,
    Down,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FeedbackView {
    pub id: ActionHash,
    pub kind: FeedbackKind,
    pub title: String,
    pub description: String,
    pub author: AgentPubKey,
    pub created_at: Timestamp,
    pub upvotes: u32,
    pub downvotes: u32,
    pub my_vote: Option<VoteDirection>,
    /// The newest of the item's creation and any vote (web's trending clock).
    pub last_activity: Timestamp,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FeedbackVote {
    pub id: ActionHash,
    /// `None` clears the caller's vote.
    pub direction: Option<VoteDirection>,
}

pub fn create(input: Feedback) -> ExternResult<FeedbackView> {
    let item = Feedback {
        kind: input.kind,
        title: input.title.trim().to_string(),
        description: input.description.trim().to_string(),
    };
    let hash = create_entry(&EntryTypes::Feedback(item))?;
    let anchor = Path::from(ALL_FEEDBACK).path_entry_hash()?;
    create_link(anchor, hash.clone(), LinkTypes::AllFeedback, ())?;
    get(hash)?.ok_or_else(|| crate::guest_error("feedback not found after writing it"))
}

pub fn list() -> ExternResult<Vec<FeedbackView>> {
    let anchor = Path::from(ALL_FEEDBACK).path_entry_hash()?;
    let mut items = Vec::new();
    for hash in action_targets(links(anchor, LinkTypes::AllFeedback)?) {
        items.extend(get(hash)?);
    }
    items.sort_by_key(|i| std::cmp::Reverse(i.created_at));
    Ok(items)
}

pub fn get(id: ActionHash) -> ExternResult<Option<FeedbackView>> {
    let Some(record) = live(id.clone())? else {
        return Ok(None);
    };
    let Some(item) = decode::<Feedback>(&record)? else {
        return Ok(None);
    };
    let me = agent_info()?.agent_initial_pubkey;
    let votes = newest_votes(&id)?;
    let count =
        |dir| u32::try_from(votes.values().filter(|(d, _)| *d == dir).count()).unwrap_or(u32::MAX);
    let created_at = record.action().timestamp();
    Ok(Some(FeedbackView {
        id,
        kind: item.kind,
        title: item.title,
        description: item.description,
        author: record.action().author().clone(),
        created_at,
        upvotes: count(VoteDirection::Up),
        downvotes: count(VoteDirection::Down),
        my_vote: votes.get(&me).map(|(d, _)| *d),
        last_activity: votes.values().map(|(_, at)| *at).fold(created_at, Ord::max),
    }))
}

/// Each voter's newest vote and when they cast it. The caller's own vote comes from their
/// chain: network reads lag right after a write.
fn newest_votes(
    id: &ActionHash,
) -> ExternResult<BTreeMap<AgentPubKey, (VoteDirection, Timestamp)>> {
    let mut found = links(id.clone(), LinkTypes::FeedbackVotes)?;
    found.sort_by_key(|l| l.timestamp);
    let mut votes = BTreeMap::new();
    for link in found {
        let Some(voter) = link.target.into_agent_pub_key() else {
            continue;
        };
        if let Some(direction) = direction_of(&link.tag) {
            votes.insert(voter, (direction, link.timestamp));
        }
    }
    let me = agent_info()?.agent_initial_pubkey;
    match own_vote(id)? {
        Some(vote) => votes.insert(me, vote),
        None => votes.remove(&me),
    };
    Ok(votes)
}

fn direction_of(tag: &LinkTag) -> Option<VoteDirection> {
    match tag.0.as_slice() {
        [VOTE_UP] => Some(VoteDirection::Up),
        [VOTE_DOWN] => Some(VoteDirection::Down),
        _ => None,
    }
}

/// The caller's live vote links on `id`, straight from their chain.
fn own_vote_links(id: &ActionHash) -> ExternResult<Vec<(ActionHash, LinkTag, Timestamp)>> {
    let scoped: ScopedLinkType = LinkTypes::FeedbackVotes.try_into()?;
    let removed: BTreeSet<ActionHash> =
        query(ChainQueryFilter::new().action_type(ActionType::DeleteLink))?
            .iter()
            .filter_map(|r| match &r.action().data {
                ActionData::DeleteLink(d) => Some(d.link_add_address.clone()),
                _ => None,
            })
            .collect();
    let base = AnyLinkableHash::from(id.clone());
    Ok(
        query(ChainQueryFilter::new().action_type(ActionType::CreateLink))?
            .iter()
            .filter(|r| !removed.contains(r.action_address()))
            .filter_map(|r| match &r.action().data {
                ActionData::CreateLink(c)
                    if c.zome_index == scoped.zome_index
                        && c.link_type == scoped.zome_type
                        && c.base_address == base =>
                {
                    Some((
                        r.action_address().clone(),
                        c.tag.clone(),
                        r.action().timestamp(),
                    ))
                }
                _ => None,
            })
            .collect(),
    )
}

fn own_vote(id: &ActionHash) -> ExternResult<Option<(VoteDirection, Timestamp)>> {
    Ok(own_vote_links(id)?
        .into_iter()
        .max_by_key(|(_, _, at)| *at)
        .and_then(|(_, tag, at)| Some((direction_of(&tag)?, at))))
}

/// Replace the caller's vote on an item (or clear it).
pub fn vote(input: FeedbackVote) -> ExternResult<Option<FeedbackView>> {
    let me = agent_info()?.agent_initial_pubkey;
    for (hash, _, _) in own_vote_links(&input.id)? {
        delete_link(hash, GetOptions::default())?;
    }
    if let Some(direction) = input.direction {
        let tag = if direction == VoteDirection::Up {
            VOTE_UP
        } else {
            VOTE_DOWN
        };
        create_link(
            input.id.clone(),
            me,
            LinkTypes::FeedbackVotes,
            LinkTag::new(vec![tag]),
        )?;
    }
    get(input.id)
}
