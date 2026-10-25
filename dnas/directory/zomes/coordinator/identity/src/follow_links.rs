//! Reading follow links. A person's own links come from their chain (network reads lag right
//! after a write); links others wrote come from the network.

use std::collections::BTreeSet;

use hdk::prelude::*;
use identity_integrity::LinkTypes;
use sp_follow::Facts;

use crate::records::links;

/// One live follow link: who it points at, when it was made, and its hash (for deleting).
#[derive(Debug, Clone)]
pub struct Edge {
    pub other: AgentPubKey,
    pub at: i64,
    pub hash: ActionHash,
}

/// Live links of `link_type` from `base`. `Followers` links are written by the follower, so
/// they always come from the network; the other kinds are written by `base` itself.
pub fn edges(base: &AgentPubKey, link_type: LinkTypes) -> ExternResult<Vec<Edge>> {
    let me = agent_info()?.agent_initial_pubkey;
    if base == &me && !matches!(link_type, LinkTypes::Followers) {
        return own_edges(link_type);
    }
    Ok(links(base.clone(), link_type)?
        .into_iter()
        .filter_map(|l| {
            Some(Edge {
                other: l.target.into_agent_pub_key()?,
                at: l.timestamp.as_micros(),
                hash: l.create_link_hash,
            })
        })
        .collect())
}

/// The caller's own live links of `link_type`, straight from their chain. `other` is the
/// person the link is about: the base for `Followers` (which point at their author).
pub fn own_edges(link_type: LinkTypes) -> ExternResult<Vec<Edge>> {
    let about_base = matches!(link_type, LinkTypes::Followers);
    let scoped: ScopedLinkType = link_type.try_into()?;
    let removed: BTreeSet<ActionHash> =
        query(ChainQueryFilter::new().action_type(ActionType::DeleteLink))?
            .iter()
            .filter_map(|r| match &r.action().data {
                ActionData::DeleteLink(d) => Some(d.link_add_address.clone()),
                _ => None,
            })
            .collect();
    let creates = query(ChainQueryFilter::new().action_type(ActionType::CreateLink))?;
    Ok(creates
        .iter()
        .filter(|r| !removed.contains(r.action_address()))
        .filter_map(|r| match &r.action().data {
            ActionData::CreateLink(c)
                if c.zome_index == scoped.zome_index && c.link_type == scoped.zome_type =>
            {
                let other = if about_base {
                    &c.base_address
                } else {
                    &c.target_address
                };
                Some(Edge {
                    other: other.clone().into_agent_pub_key()?,
                    at: r.action().timestamp().as_micros(),
                    hash: r.action_address().clone(),
                })
            }
            _ => None,
        })
        .collect())
}

/// What a person has decided about the people following them.
#[derive(Debug, Default)]
pub struct Policy {
    pub approval_since: Option<i64>,
    accepts: Vec<Edge>,
    rejects: Vec<Edge>,
}

impl Policy {
    pub fn of(person: &AgentPubKey) -> ExternResult<Self> {
        Ok(Self {
            approval_since: edges(person, LinkTypes::RequiresFollowApproval)?
                .iter()
                .map(|e| e.at)
                .min(),
            accepts: edges(person, LinkTypes::FollowAccepts)?,
            rejects: edges(person, LinkTypes::FollowRejects)?,
        })
    }

    pub fn status(&self, follower: &AgentPubKey, followed_at: i64) -> sp_follow::Status {
        sp_follow::status(&Facts {
            followed_at,
            approval_since: self.approval_since,
            accepted_at: self.accepted_at(follower),
            rejected_at: newest(&self.rejects, follower),
        })
    }

    /// When the person last accepted `follower`'s request, if ever.
    pub fn accepted_at(&self, follower: &AgentPubKey) -> Option<i64> {
        newest(&self.accepts, follower)
    }
}

fn newest(list: &[Edge], who: &AgentPubKey) -> Option<i64> {
    list.iter().filter(|e| &e.other == who).map(|e| e.at).max()
}

/// Keep the newest edge per person (someone who followed twice counts once).
pub fn newest_per_person(mut list: Vec<Edge>) -> Vec<Edge> {
    list.sort_by_key(|e| std::cmp::Reverse(e.at));
    let mut seen = BTreeSet::new();
    list.retain(|e| seen.insert(e.other.clone()));
    list
}
