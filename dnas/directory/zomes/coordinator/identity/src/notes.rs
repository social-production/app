//! A profile's cover photo and the notes that list someone's comments (Profiles and Comments
//! mockups §2–3). Each is linked from its author's key; the caller's own come from their chain
//! too, since network reads lag a write.

use hdk::prelude::*;
use identity_integrity::{CommentNote, Cover, EntryTypes, LinkTypes};

use crate::records::{decode, links, live};

/// How many of someone's newest comments a read returns.
const NOTES_MAX: usize = 50;

#[derive(Serialize, Deserialize, Debug)]
pub struct CommentNoteView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub created_at: Timestamp,
    pub note: CommentNote,
}

/// Replace the caller's cover (or remove it with `None`).
#[hdk_extern]
pub fn set_cover(cover: Option<Cover>) -> ExternResult<Option<ActionHash>> {
    let me = agent_info()?.agent_initial_pubkey;
    for (link, target) in own_links(LinkTypes::AgentToCover)? {
        delete_link(link, GetOptions::default())?;
        delete_entry(target)?;
    }
    let Some(cover) = cover else { return Ok(None) };
    let hash = create_entry(&EntryTypes::Cover(cover))?;
    create_link(me, hash.clone(), LinkTypes::AgentToCover, ())?;
    Ok(Some(hash))
}

/// Someone's cover photo, if they have one.
#[hdk_extern]
pub fn get_cover(agent: AgentPubKey) -> ExternResult<Option<Cover>> {
    let ids = if agent == agent_info()?.agent_initial_pubkey {
        own_links(LinkTypes::AgentToCover)?
            .into_iter()
            .map(|(_, t)| t)
            .collect()
    } else {
        newest_targets(agent, LinkTypes::AgentToCover)?
    };
    for id in ids {
        if let Some(record) = live(id)? {
            return decode::<Cover>(&record);
        }
    }
    Ok(None)
}

/// Note a comment the caller just wrote in a space's chat.
#[hdk_extern]
pub fn note_comment(note: CommentNote) -> ExternResult<ActionHash> {
    let me = agent_info()?.agent_initial_pubkey;
    let hash = create_entry(&EntryTypes::CommentNote(note))?;
    create_link(me, hash.clone(), LinkTypes::AgentToComments, ())?;
    Ok(hash)
}

/// Take back the note for a comment the caller deleted.
#[hdk_extern]
pub fn unnote_comment(message: ActionHash) -> ExternResult<()> {
    for (link, target) in own_links(LinkTypes::AgentToComments)? {
        let Some(record) = live(target.clone())? else {
            continue;
        };
        if decode::<CommentNote>(&record)?.is_some_and(|n| n.message == message) {
            delete_link(link, GetOptions::default())?;
            delete_entry(target)?;
        }
    }
    Ok(())
}

/// Someone's newest comments, newest first.
#[hdk_extern]
pub fn comments_of(agent: AgentPubKey) -> ExternResult<Vec<CommentNoteView>> {
    let mut ids: Vec<ActionHash> = newest_targets(agent.clone(), LinkTypes::AgentToComments)?;
    if agent == agent_info()?.agent_initial_pubkey {
        for (_, id) in own_links(LinkTypes::AgentToComments)? {
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }
    let mut out = Vec::new();
    for id in ids {
        let Some(record) = live(id.clone())? else {
            continue;
        };
        if let Some(note) = decode::<CommentNote>(&record)? {
            out.push(CommentNoteView {
                id,
                author: record.action().author().clone(),
                created_at: record.action().timestamp(),
                note,
            });
        }
    }
    out.sort_by_key(|v| std::cmp::Reverse(v.created_at));
    out.truncate(NOTES_MAX);
    Ok(out)
}

fn newest_targets(agent: AgentPubKey, link_type: LinkTypes) -> ExternResult<Vec<ActionHash>> {
    let mut found = links(agent, link_type)?;
    found.sort_by_key(|l| std::cmp::Reverse(l.timestamp));
    Ok(found
        .into_iter()
        .filter_map(|l| l.target.into_action_hash())
        .collect())
}

/// The caller's live links of `link_type` from their own key, newest first: (link, target).
fn own_links(link_type: LinkTypes) -> ExternResult<Vec<(ActionHash, ActionHash)>> {
    let scoped: ScopedLinkType = link_type.try_into()?;
    let me = AnyLinkableHash::from(agent_info()?.agent_initial_pubkey);
    let removed: Vec<ActionHash> =
        query(ChainQueryFilter::new().action_type(ActionType::DeleteLink))?
            .iter()
            .filter_map(|r| match &r.action().data {
                ActionData::DeleteLink(d) => Some(d.link_add_address.clone()),
                _ => None,
            })
            .collect();
    let mut out: Vec<(ActionHash, ActionHash)> =
        query(ChainQueryFilter::new().action_type(ActionType::CreateLink))?
            .iter()
            .filter(|r| !removed.contains(r.action_address()))
            .filter_map(|r| match &r.action().data {
                ActionData::CreateLink(c)
                    if c.zome_index == scoped.zome_index
                        && c.link_type == scoped.zome_type
                        && c.base_address == me =>
                {
                    c.target_address
                        .clone()
                        .into_action_hash()
                        .map(|t| (r.action_address().clone(), t))
                }
                _ => None,
            })
            .collect();
    out.reverse();
    Ok(out)
}
