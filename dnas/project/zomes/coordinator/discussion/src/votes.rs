//! Comment votes (ADR 0070): web's up/down `VoteStrip` on a thread's or post's comments. A vote
//! is a `MessageVotes` link from the comment to the voter; changing it removes the old one.

use discussion_integrity::LinkTypes;
use hdk::prelude::*;
use sp_discussion::{direction_of, tag_of};

use crate::{guest, reads};

#[derive(Serialize, Deserialize, Debug)]
pub struct VoteInput {
    pub message: ActionHash,
    /// web's direction: 1 up, -1 down, 0 takes the vote back.
    pub direction: i8,
}

/// A comment's score (ups minus downs) and the caller's own vote.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VoteView {
    pub id: ActionHash,
    pub count: i32,
    pub mine: i8,
}

fn links_on(message: &ActionHash) -> ExternResult<Vec<Link>> {
    get_links(
        LinkQuery::try_new(message.clone(), LinkTypes::MessageVotes)?,
        GetStrategy::default(),
    )
}

pub fn set(input: &VoteInput) -> ExternResult<()> {
    if reads::record(&input.message)?.is_none() {
        return Err(guest("This comment isn't here yet. Try again in a moment."));
    }
    let me = agent_info()?.agent_initial_pubkey;
    for link in links_on(&input.message)? {
        if link.author == me {
            delete_link(link.create_link_hash, GetOptions::default())?;
        }
    }
    if let Some(tag) = tag_of(input.direction) {
        create_link(
            input.message.clone(),
            me,
            LinkTypes::MessageVotes,
            LinkTag::new(vec![tag]),
        )?;
    }
    Ok(())
}

/// One vote per person counts (their newest), as web's `content_votes` keeps one row each.
pub fn of(messages: &[ActionHash]) -> ExternResult<Vec<VoteView>> {
    let me = agent_info()?.agent_initial_pubkey;
    let mut out = Vec::with_capacity(messages.len());
    for id in messages {
        let mut links = links_on(id)?;
        links.sort_by_key(|l| std::cmp::Reverse(l.timestamp));
        let mut seen = Vec::new();
        let (mut count, mut mine) = (0i32, 0i8);
        for link in links {
            if seen.contains(&link.author) {
                continue;
            }
            let direction = direction_of(&link.tag.0);
            count += i32::from(direction);
            if link.author == me {
                mine = direction;
            }
            seen.push(link.author);
        }
        out.push(VoteView {
            id: id.clone(),
            count,
            mine,
        });
    }
    Ok(out)
}
