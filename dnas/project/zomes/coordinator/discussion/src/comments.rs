//! Comments in a thread (web's `comments` with `subject_type = thread`, ADR 0041): anyone signed
//! in comments or replies; never edited. Reads merge the thread's links with the caller's own
//! chain, so your comment shows at once.

use std::collections::{BTreeMap, BTreeSet};

use discussion_integrity::{
    decode_comment, is_thread, Comment, EntryTypes, LinkTypes, UnitEntryTypes,
};
use hdk::prelude::*;
use sp_core::discussion::{check_comment, COMMENT_PAGE_MAX};

use crate::votes::{many_links, scores};
use crate::{guest, own, reads};

#[derive(Serialize, Deserialize, Debug)]
pub struct CommentInput {
    pub thread: ActionHash,
    pub parent: Option<ActionHash>,
    pub body: String,
}

/// One comment as the thread page shows it; the UI nests replies under `parent`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommentView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub parent: Option<ActionHash>,
    pub body: String,
    pub created_at: Timestamp,
    pub vote_count: i32,
    pub my_vote: i8,
}

#[hdk_extern]
pub fn add_comment(input: CommentInput) -> ExternResult<ActionHash> {
    let body = input.body.trim().to_string();
    check_comment(&body).map_err(|reason| guest(&reason))?;
    let found = match reads::record(&input.thread)? {
        Some(r) => is_thread(r.action())?,
        None => false,
    };
    if !found {
        return Err(guest("This thread isn't here yet. Try again in a moment."));
    }
    if let Some(parent) = &input.parent {
        let found = reads::record(parent)?;
        let Some(original) = found.as_ref().map(decode_comment).transpose()?.flatten() else {
            return Err(guest(
                "The comment you are replying to isn't here yet. Try again in a moment.",
            ));
        };
        if original.thread != input.thread {
            return Err(guest("A reply answers a comment in the same thread."));
        }
    }
    let comment = Comment {
        thread: input.thread.clone(),
        parent: input.parent,
        body,
    };
    let hash = create_entry(EntryTypes::Comment(comment))?;
    create_link(input.thread, hash.clone(), LinkTypes::ThreadComments, ())?;
    Ok(hash)
}

/// The comments of each thread (network links plus the caller's own), in the same order.
pub fn comment_sets(threads: &[ActionHash]) -> ExternResult<Vec<BTreeSet<ActionHash>>> {
    let deleted = own::deleted()?;
    let mut mine: BTreeMap<ActionHash, BTreeSet<ActionHash>> = BTreeMap::new();
    for r in own::entries(UnitEntryTypes::Comment)? {
        if let Some(c) = decode_comment(&r)? {
            mine.entry(c.thread)
                .or_default()
                .insert(r.action_address().clone());
        }
    }
    let links = many_links(threads, LinkTypes::ThreadComments)?;
    Ok(threads
        .iter()
        .zip(links)
        .map(|(thread, links)| {
            let mut set: BTreeSet<ActionHash> = links
                .into_iter()
                .filter_map(|l| l.target.into_action_hash())
                .collect();
            set.extend(mine.remove(thread).unwrap_or_default());
            set.retain(|hash| !deleted.contains(hash));
            set
        })
        .collect())
}

/// A thread's comments, oldest first: the first `COMMENT_PAGE_MAX`, as web's `get_comments`.
pub fn list(thread: &ActionHash) -> ExternResult<Vec<CommentView>> {
    let hashes: Vec<ActionHash> = comment_sets(std::slice::from_ref(thread))?
        .into_iter()
        .flatten()
        .collect();
    let mut found = Vec::new();
    for r in reads::records(&hashes)?.into_iter().flatten() {
        if let Some(c) = decode_comment(&r)? {
            if &c.thread == thread {
                found.push((r, c));
            }
        }
    }
    found.sort_by_key(|(r, _)| r.action().timestamp());
    found.truncate(COMMENT_PAGE_MAX);
    let ids: Vec<ActionHash> = found
        .iter()
        .map(|(r, _)| r.action_address().clone())
        .collect();
    let scores = scores(&ids)?;
    Ok(found
        .into_iter()
        .zip(scores)
        .map(|((r, c), score)| CommentView {
            id: r.action_address().clone(),
            author: r.action().author().clone(),
            parent: c.parent,
            body: c.body,
            created_at: r.action().timestamp(),
            vote_count: score.vote_count,
            my_vote: score.my_vote,
        })
        .collect())
}
