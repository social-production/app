//! A post's photos (ADR 0101). A post's space keeps them as one chat message by the post's
//! author, listed in the space's chat with the tag `media` so it is not one of the comments.
//! No integrity change: the message, its parts and the link are the chat's own (ADR 0056).

use std::collections::BTreeSet;

use discussion_integrity::{Attachment, ChatMessage, EntryTypes, LinkTypes};
use hdk::prelude::*;

use crate::{anchor, files, guest, own, reads, scope};

/// The tag of the chat link that lists a post's photos.
pub const TAG: &[u8] = b"media";
/// Photos on one post (Twitter's four).
pub const PHOTOS_MAX: usize = 4;

/// The space's founder: the author of its charter.
fn founder(space: &ActionHash) -> ExternResult<Option<AgentPubKey>> {
    Ok(reads::record(space)?.map(|r| r.action().author().clone()))
}

/// Add the photos to your own post, once.
pub fn set(attachments: Vec<Attachment>) -> ExternResult<ActionHash> {
    let photos = attachments.iter().all(|a| a.kind == "image");
    if attachments.is_empty() || attachments.len() > PHOTOS_MAX || !photos {
        return Err(guest(&format!("A post carries up to {PHOTOS_MAX} photos.")));
    }
    files::check(&attachments)?;
    let space = anchor()?;
    if founder(&space)? != Some(agent_info()?.agent_initial_pubkey) {
        return Err(guest("Only a post's author adds its photos."));
    }
    if !own::media()?.is_empty() {
        return Err(guest("This post already has its photos."));
    }
    let message = ChatMessage {
        space: space.clone(),
        body: String::new(),
        reply_to: None,
        attachments,
    };
    let hash = create_entry(EntryTypes::ChatMessage(message))?;
    create_link(
        space,
        hash.clone(),
        LinkTypes::ChatMessages,
        LinkTag::new(TAG),
    )?;
    scope::touch()?;
    Ok(hash)
}

/// Whether a chat link lists photos rather than a comment.
pub fn is_media(link: &Link) -> bool {
    link.tag.0 == TAG
}

/// The space's photos: the founder's media message, the caller's own copy first.
pub fn get() -> ExternResult<Vec<Attachment>> {
    let space = anchor()?;
    let Some(founder) = founder(&space)? else {
        return Ok(Vec::new());
    };
    let mut ids: Vec<ActionHash> = Vec::new();
    if founder == agent_info()?.agent_initial_pubkey {
        ids.extend(own::media()?);
    }
    let mut links = get_links(
        LinkQuery::try_new(space, LinkTypes::ChatMessages)?,
        GetStrategy::default(),
    )?;
    links.sort_by_key(|l| l.timestamp);
    ids.extend(
        links
            .into_iter()
            .filter(|l| is_media(l) && l.author == founder)
            .filter_map(|l| l.target.into_action_hash()),
    );
    let deleted = own::deleted()?;
    for id in ids {
        if deleted.contains(&id) {
            continue;
        }
        let Some(record) = reads::record(&id)? else {
            continue;
        };
        if record.action().author() != &founder {
            continue;
        }
        if let Some(message) = reads::decode(&record)? {
            return Ok(message.attachments);
        }
    }
    Ok(Vec::new())
}

/// The caller's own media messages in this space, which are not comments.
pub fn own_ids() -> ExternResult<BTreeSet<ActionHash>> {
    Ok(own::media()?.into_iter().collect())
}
