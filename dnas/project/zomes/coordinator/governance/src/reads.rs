//! Reading proposals, votes and outcomes off the DHT, plus the caller's own fresh writes.

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::{LinkTypes, Outcome, Proposal, Vote};

pub const PROPOSALS: &str = "proposals";

pub fn anchor() -> ExternResult<EntryHash> {
    Path::from(PROPOSALS).path_entry_hash()
}

fn targets(
    base: impl Into<AnyLinkableHash>,
    link_type: LinkTypes,
) -> ExternResult<Vec<(Timestamp, ActionHash)>> {
    let links = get_links(LinkQuery::try_new(base, link_type)?, GetStrategy::default())?;
    Ok(links
        .into_iter()
        .filter_map(|l| l.target.into_action_hash().map(|h| (l.timestamp, h)))
        .collect())
}

fn decode<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
    record: &Record,
) -> ExternResult<Option<T>> {
    record
        .entry()
        .to_app_option::<T>()
        .map_err(|e| wasm_error!(e))
}

pub struct Found<T> {
    pub hash: ActionHash,
    pub author: AgentPubKey,
    pub at: Timestamp,
    pub entry: T,
}

pub fn found<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
    hash: ActionHash,
) -> ExternResult<Option<Found<T>>> {
    let Some(record) = get(hash.clone(), GetOptions::default())? else {
        return Ok(None);
    };
    let author = record.action().author().clone();
    let at = record.action().timestamp();
    Ok(decode::<T>(&record)?.map(|entry| Found {
        hash,
        author,
        at,
        entry,
    }))
}

/// Every proposal, oldest first.
pub fn proposals() -> ExternResult<Vec<Found<Proposal>>> {
    let mut hashes = targets(anchor()?, LinkTypes::Proposals)?;
    hashes.sort();
    hashes.dedup_by(|a, b| a.1 == b.1);
    hashes
        .into_iter()
        .filter_map(|(_, h)| found::<Proposal>(h).transpose())
        .collect()
}

pub fn proposal(hash: &ActionHash) -> ExternResult<Option<Found<Proposal>>> {
    found::<Proposal>(hash.clone())
}

/// Each voter's first vote on `proposal` (votes are final; a second one never counts).
pub fn votes(proposal: &ActionHash) -> ExternResult<Vec<Found<Vote>>> {
    let mut first: BTreeMap<AgentPubKey, Found<Vote>> = BTreeMap::new();
    for (_, hash) in targets(proposal.clone(), LinkTypes::ProposalToVotes)? {
        let Some(vote) = found::<Vote>(hash)? else {
            continue;
        };
        if vote.entry.proposal != *proposal {
            continue;
        }
        let keep = first.get(&vote.author).is_none_or(|v| vote.at < v.at);
        if keep {
            first.insert(vote.author.clone(), vote);
        }
    }
    Ok(first.into_values().collect())
}

/// The earliest outcome written for `proposal`.
pub fn outcome(proposal: &ActionHash) -> ExternResult<Option<Found<Outcome>>> {
    let mut found_outcomes = Vec::new();
    for (_, hash) in targets(proposal.clone(), LinkTypes::ProposalToOutcomes)? {
        if let Some(o) = found::<Outcome>(hash)? {
            found_outcomes.push(o);
        }
    }
    Ok(found_outcomes.into_iter().min_by_key(|o| o.at))
}
