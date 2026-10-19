//! Up/down votes on threads and comments (web's `content_votes`, ADR 0041): one live vote per
//! agent and target. Others' votes come from the network (each voter's latest link), the
//! caller's own from its chain, so your vote shows at once.

use std::collections::BTreeMap;

use discussion_integrity::{is_comment, is_thread, LinkTypes};
use hdk::prelude::*;
use sp_core::discussion::{tally, vote_direction, vote_tag};

use crate::{guest, own, reads};

#[derive(Serialize, Deserialize, Debug)]
pub struct VoteInput {
    pub target: ActionHash,
    /// 1 up, -1 down, 0 takes the vote back (web's up / down / neutral).
    pub direction: i8,
}

/// A target's score and the caller's own vote on it.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, Default)]
pub struct Score {
    pub vote_count: i32,
    pub my_vote: i8,
}

/// Links of `link_type` from many bases in one host call.
pub fn many_links(bases: &[ActionHash], link_type: LinkTypes) -> ExternResult<Vec<Vec<Link>>> {
    if bases.is_empty() {
        return Ok(Vec::new());
    }
    let inputs = bases
        .iter()
        .map(|base| {
            let query = LinkQuery::try_new(base.clone(), link_type)?;
            Ok(GetLinksInput::from_query(query, GetStrategy::default()))
        })
        .collect::<ExternResult<Vec<_>>>()?;
    HDK.with(|h| h.borrow().get_links(inputs))
}

/// Replace the caller's vote on a thread or comment; direction 0 withdraws it.
#[hdk_extern]
pub fn cast_vote(input: VoteInput) -> ExternResult<()> {
    if !(-1..=1).contains(&input.direction) {
        return Err(guest("A vote is up, down or neutral."));
    }
    let Some(record) = reads::record(&input.target)? else {
        return Err(guest("That isn't here yet. Try again in a moment."));
    };
    if !is_thread(record.action())? && !is_comment(record.action())? {
        return Err(guest("Votes go on threads and comments."));
    }
    let base = AnyLinkableHash::from(input.target.clone());
    for link in own::live_links(LinkTypes::Votes)? {
        if link.base == base {
            delete_link(link.hash, GetOptions::default())?;
        }
    }
    if let Some(tag) = vote_tag(input.direction) {
        let me = agent_info()?.agent_initial_pubkey;
        create_link(input.target, me, LinkTypes::Votes, LinkTag::new([tag]))?;
    }
    Ok(())
}

/// Scores for many targets, in the same order.
pub fn scores(targets: &[ActionHash]) -> ExternResult<Vec<Score>> {
    let me = agent_info()?.agent_initial_pubkey;
    // Oldest first, so a later live vote (should two ever exist) wins.
    let mine: BTreeMap<AnyLinkableHash, i8> = own::live_links(LinkTypes::Votes)?
        .into_iter()
        .map(|l| (l.base, vote_direction(&l.tag.0)))
        .collect();
    let links = many_links(targets, LinkTypes::Votes)?;
    Ok(targets
        .iter()
        .zip(links)
        .map(|(target, links)| {
            let mut latest: BTreeMap<AgentPubKey, (Timestamp, i8)> = BTreeMap::new();
            for l in links.into_iter().filter(|l| l.author != me) {
                let vote = (l.timestamp, vote_direction(&l.tag.0));
                let entry = latest.entry(l.author).or_insert(vote);
                if vote.0 >= entry.0 {
                    *entry = vote;
                }
            }
            let my_vote = mine
                .get(&AnyLinkableHash::from(target.clone()))
                .copied()
                .unwrap_or(0);
            Score {
                vote_count: tally(latest.values().map(|v| v.1).chain([my_vote])),
                my_vote,
            }
        })
        .collect())
}
