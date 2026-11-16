//! Comments on platform feedback (Plan, Phase and Feedback Screens §7): flat, each may name
//! the comment it answers; likes are links from a comment to whoever likes it. The caller's
//! own comments and likes also come from their chain, since network reads lag a write.

use std::collections::BTreeSet;

use hdk::prelude::*;
use identity_integrity::{EntryTypes, FeedbackComment, LinkTypes};

use crate::records::{action_targets, decode, links, live};

#[derive(Serialize, Deserialize, Debug)]
pub struct FeedbackCommentView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub body: String,
    pub reply_to: Option<ActionHash>,
    pub created_at: Timestamp,
    pub likes: u32,
    pub liked: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewFeedbackComment {
    pub feedback: ActionHash,
    pub body: String,
    pub reply_to: Option<ActionHash>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FeedbackCommentLike {
    pub id: ActionHash,
    pub like: bool,
}

pub fn add(input: NewFeedbackComment) -> ExternResult<FeedbackCommentView> {
    let comment = FeedbackComment {
        feedback: input.feedback.clone(),
        body: input.body.trim().to_string(),
        reply_to: input.reply_to,
    };
    let hash = create_entry(&EntryTypes::FeedbackComment(comment))?;
    create_link(
        input.feedback,
        hash.clone(),
        LinkTypes::FeedbackComments,
        (),
    )?;
    view(hash)?.ok_or_else(|| crate::guest_error("comment not found after writing it"))
}

/// An item's comments, oldest first.
pub fn list(feedback: ActionHash) -> ExternResult<Vec<FeedbackCommentView>> {
    let mut ids: BTreeSet<ActionHash> =
        action_targets(links(feedback.clone(), LinkTypes::FeedbackComments)?)
            .into_iter()
            .collect();
    ids.extend(own_link_targets(&feedback, LinkTypes::FeedbackComments)?);
    let mut out = Vec::new();
    for id in ids {
        out.extend(view(id)?);
    }
    out.sort_by_key(|c| c.created_at);
    Ok(out)
}

/// How many comments an item has (its row's "3 replies").
pub fn count(feedback: ActionHash) -> ExternResult<u32> {
    let n = links(feedback, LinkTypes::FeedbackComments)?.len();
    Ok(u32::try_from(n).unwrap_or(u32::MAX))
}

fn view(id: ActionHash) -> ExternResult<Option<FeedbackCommentView>> {
    let Some(record) = live(id.clone())? else {
        return Ok(None);
    };
    let Some(comment) = decode::<FeedbackComment>(&record)? else {
        return Ok(None);
    };
    let me = agent_info()?.agent_initial_pubkey;
    let mut likers: BTreeSet<AgentPubKey> = links(id.clone(), LinkTypes::FeedbackCommentLikes)?
        .into_iter()
        .filter_map(|l| l.target.into_agent_pub_key())
        .collect();
    let mine = !own_links(&id, LinkTypes::FeedbackCommentLikes)?.is_empty();
    if mine {
        likers.insert(me);
    } else {
        likers.remove(&me);
    }
    Ok(Some(FeedbackCommentView {
        id,
        author: record.action().author().clone(),
        body: comment.body,
        reply_to: comment.reply_to,
        created_at: record.action().timestamp(),
        likes: u32::try_from(likers.len()).unwrap_or(u32::MAX),
        liked: mine,
    }))
}

/// Like or unlike a comment.
pub fn like(input: FeedbackCommentLike) -> ExternResult<Option<FeedbackCommentView>> {
    let mine = own_links(&input.id, LinkTypes::FeedbackCommentLikes)?;
    if input.like && mine.is_empty() {
        let me = agent_info()?.agent_initial_pubkey;
        create_link(input.id.clone(), me, LinkTypes::FeedbackCommentLikes, ())?;
    }
    if !input.like {
        for (hash, _) in mine {
            delete_link(hash, GetOptions::default())?;
        }
    }
    view(input.id)
}

/// The caller's live links of `link_type` from `base`, from their own chain: (link, target).
fn own_links(
    base: &ActionHash,
    link_type: LinkTypes,
) -> ExternResult<Vec<(ActionHash, AnyLinkableHash)>> {
    let scoped: ScopedLinkType = link_type.try_into()?;
    let removed: BTreeSet<ActionHash> =
        query(ChainQueryFilter::new().action_type(ActionType::DeleteLink))?
            .iter()
            .filter_map(|r| match &r.action().data {
                ActionData::DeleteLink(d) => Some(d.link_add_address.clone()),
                _ => None,
            })
            .collect();
    let base = AnyLinkableHash::from(base.clone());
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
                    Some((r.action_address().clone(), c.target_address.clone()))
                }
                _ => None,
            })
            .collect(),
    )
}

fn own_link_targets(base: &ActionHash, link_type: LinkTypes) -> ExternResult<Vec<ActionHash>> {
    Ok(own_links(base, link_type)?
        .into_iter()
        .filter_map(|(_, t)| t.into_action_hash())
        .collect())
}
