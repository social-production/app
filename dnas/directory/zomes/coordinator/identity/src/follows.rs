//! Follows (ADR 0054). A follow is accepted at once, or pending while the person followed
//! requires approval, until they accept or reject it. The status rule is `sp_follow::status`.

use hdk::prelude::*;
use identity_integrity::LinkTypes;

use crate::follow_links::{edges, newest_per_person, own_edges, Edge, Policy};
use crate::guest_error;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FollowStatus {
    Accepted,
    Pending,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FollowRow {
    pub agent: AgentPubKey,
    pub status: FollowStatus,
    pub since: Timestamp,
    /// When an approval-gated follow was accepted (for the follower's notification).
    pub accepted_at: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FollowView {
    pub requires_approval: bool,
    /// People following this person (pending ones too; the UI shows those only to them).
    pub followers: Vec<FollowRow>,
    /// People this person follows.
    pub following: Vec<FollowRow>,
    /// The caller's own follow of this person, if any.
    pub my_status: Option<FollowStatus>,
}

fn visible(status: sp_follow::Status) -> Option<FollowStatus> {
    match status {
        sp_follow::Status::Accepted => Some(FollowStatus::Accepted),
        sp_follow::Status::Pending => Some(FollowStatus::Pending),
        sp_follow::Status::Rejected => None,
    }
}

fn row(edge: &Edge, status: sp_follow::Status, accepted_at: Option<i64>) -> Option<FollowRow> {
    let status = visible(status)?;
    Some(FollowRow {
        agent: edge.other.clone(),
        status,
        since: Timestamp::from_micros(edge.at),
        accepted_at: accepted_at
            .filter(|&at| status == FollowStatus::Accepted && at >= edge.at)
            .map(Timestamp::from_micros),
    })
}

fn my_follow_of(target: &AgentPubKey) -> ExternResult<Vec<Edge>> {
    Ok(own_edges(LinkTypes::Follows)?
        .into_iter()
        .filter(|e| &e.other == target)
        .collect())
}

/// The caller's status towards `target`, if they follow them and weren't rejected.
fn my_status(target: &AgentPubKey) -> ExternResult<Option<FollowStatus>> {
    let me = agent_info()?.agent_initial_pubkey;
    let Some(at) = my_follow_of(target)?.iter().map(|e| e.at).max() else {
        return Ok(None);
    };
    Ok(visible(Policy::of(target)?.status(&me, at)))
}

pub fn follow(target: AgentPubKey) -> ExternResult<Option<FollowStatus>> {
    let me = agent_info()?.agent_initial_pubkey;
    if target == me {
        return Err(guest_error("You cannot follow yourself"));
    }
    // A rejected follow can be asked again: drop the old links so the new one is newer.
    if my_status(&target)?.is_some() {
        return my_status(&target);
    }
    unfollow(target.clone())?;
    create_link(me.clone(), target.clone(), LinkTypes::Follows, ())?;
    create_link(target.clone(), me, LinkTypes::Followers, ())?;
    my_status(&target)
}

pub fn unfollow(target: AgentPubKey) -> ExternResult<()> {
    let me = agent_info()?.agent_initial_pubkey;
    for edge in my_follow_of(&target)? {
        delete_link(edge.hash, GetOptions::default())?;
    }
    for edge in edges(&target, LinkTypes::Followers)? {
        if edge.other == me {
            delete_link(edge.hash, GetOptions::default())?;
        }
    }
    Ok(())
}

pub fn decide(follower: AgentPubKey, accept: bool) -> ExternResult<ActionHash> {
    let me = agent_info()?.agent_initial_pubkey;
    let kind = if accept {
        LinkTypes::FollowAccepts
    } else {
        LinkTypes::FollowRejects
    };
    create_link(me, follower, kind, ())
}

pub fn set_approval(on: bool) -> ExternResult<bool> {
    let me = agent_info()?.agent_initial_pubkey;
    let current = own_edges(LinkTypes::RequiresFollowApproval)?;
    if on && current.is_empty() {
        create_link(me.clone(), me, LinkTypes::RequiresFollowApproval, ())?;
    }
    if !on {
        for edge in current {
            delete_link(edge.hash, GetOptions::default())?;
        }
    }
    Ok(on)
}

fn followers(person: &AgentPubKey, policy: &Policy) -> ExternResult<Vec<FollowRow>> {
    let list = newest_per_person(edges(person, LinkTypes::Followers)?);
    Ok(list
        .iter()
        .filter_map(|e| {
            row(
                e,
                policy.status(&e.other, e.at),
                policy.accepted_at(&e.other),
            )
        })
        .collect())
}

fn following(person: &AgentPubKey) -> ExternResult<Vec<FollowRow>> {
    let mut rows = Vec::new();
    for edge in newest_per_person(edges(person, LinkTypes::Follows)?) {
        let policy = Policy::of(&edge.other)?;
        let status = policy.status(person, edge.at);
        rows.extend(row(&edge, status, policy.accepted_at(person)));
    }
    Ok(rows)
}

pub fn view(person: AgentPubKey) -> ExternResult<FollowView> {
    let policy = Policy::of(&person)?;
    Ok(FollowView {
        requires_approval: policy.approval_since.is_some(),
        followers: followers(&person, &policy)?,
        following: following(&person)?,
        my_status: my_status(&person)?,
    })
}

/// Follows of the caller waiting for their decision, oldest first.
pub fn requests() -> ExternResult<Vec<FollowRow>> {
    let me = agent_info()?.agent_initial_pubkey;
    let mut rows: Vec<FollowRow> = followers(&me, &Policy::of(&me)?)?
        .into_iter()
        .filter(|r| r.status == FollowStatus::Pending)
        .collect();
    rows.sort_by_key(|r| r.since);
    Ok(rows)
}
