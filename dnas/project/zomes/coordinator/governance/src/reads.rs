//! Reading proposals, votes, outcomes and values off the DHT, plus the caller's own fresh writes.

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::{Challenge, LinkTypes, Outcome, Proposal, Rating, UnitEntryTypes, Vote};

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

/// Every outcome written for `proposal`, earliest first. Which one decides it depends on its
/// challenges (`count`, ADR 0040).
pub fn outcomes(proposal: &ActionHash) -> ExternResult<Vec<Found<Outcome>>> {
    let mut out = Vec::new();
    for (_, hash) in targets(proposal.clone(), LinkTypes::ProposalToOutcomes)? {
        if let Some(o) = found::<Outcome>(hash)? {
            if o.entry.proposal == *proposal {
                out.push(o);
            }
        }
    }
    out.sort_by(|a, b| (a.at, &a.hash).cmp(&(b.at, &b.hash)));
    out.dedup_by(|a, b| a.hash == b.hash);
    Ok(out)
}

/// Every rating on `proposal`: the network's, plus the caller's own (fresh writes lag).
pub fn ratings(proposal: &ActionHash) -> ExternResult<Vec<Found<Rating>>> {
    let all = linked::<Rating>(
        proposal.clone(),
        LinkTypes::ProposalToRatings,
        UnitEntryTypes::Rating,
    )?;
    Ok(all
        .into_iter()
        .filter(|r| r.entry.proposal == *proposal)
        .collect())
}

/// Every challenge to `proposal`'s count, the caller's own included.
pub fn challenges(proposal: &ActionHash) -> ExternResult<Vec<Found<Challenge>>> {
    let all = linked::<Challenge>(
        proposal.clone(),
        LinkTypes::ProposalToChallenges,
        UnitEntryTypes::Challenge,
    )?;
    Ok(all
        .into_iter()
        .filter(|c| c.entry.proposal == *proposal)
        .collect())
}

/// Records linked from `base`, plus the caller's own records of that type.
pub fn linked<T: TryFrom<SerializedBytes, Error = SerializedBytesError>>(
    base: impl Into<AnyLinkableHash>,
    link_type: LinkTypes,
    entry_type: UnitEntryTypes,
) -> ExternResult<Vec<Found<T>>> {
    let mut hashes: Vec<ActionHash> = targets(base, link_type)?
        .into_iter()
        .map(|(_, h)| h)
        .collect();
    let mine = query(
        ChainQueryFilter::new()
            .entry_type(EntryType::App(entry_type.try_into()?))
            .include_entries(true),
    )?;
    hashes.extend(mine.iter().map(|r| r.action_address().clone()));
    hashes.sort();
    hashes.dedup();
    hashes
        .into_iter()
        .filter_map(|h| found::<T>(h).transpose())
        .collect()
}
