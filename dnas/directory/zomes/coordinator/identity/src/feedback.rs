//! Platform feedback (ADR 0054). Items hang off a "feedback" anchor; votes are links from an
//! item to the voter, the newest per voter counting. Sorting (web's trending rule) is the UI's.

use std::collections::BTreeMap;

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

/// Each voter's newest vote and when they cast it.
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
        let direction = match link.tag.0.as_slice() {
            [VOTE_UP] => VoteDirection::Up,
            [VOTE_DOWN] => VoteDirection::Down,
            _ => continue,
        };
        votes.insert(voter, (direction, link.timestamp));
    }
    Ok(votes)
}

/// Replace the caller's vote on an item (or clear it).
pub fn vote(input: FeedbackVote) -> ExternResult<Option<FeedbackView>> {
    let me = agent_info()?.agent_initial_pubkey;
    for link in links(input.id.clone(), LinkTypes::FeedbackVotes)? {
        if link.author == me {
            delete_link(link.create_link_hash, GetOptions::default())?;
        }
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
