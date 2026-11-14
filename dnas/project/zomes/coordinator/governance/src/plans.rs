//! Plan selection (ADR 0039): approved plans are passed `Plan` proposals; the leader of each
//! kind is picked from them by approval ratio, then by members' average rating
//! (`sp_governance::plans::pick_leader`, web's `sync_project_plan_leading_flags`).

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::{ProposalKind, Rating};
use sp_governance::phases::{Phase, PROPOSAL};
use sp_governance::plans::{pick_leader, summarize, Candidate, Leader, PlanKind, Ratings, Score};

use crate::decide::{Decision, Standing};
use crate::reads::{self, Found};

/// A plan members approved (its recount passed).
pub struct Approved {
    pub hash: ActionHash,
    pub author: AgentPubKey,
    pub kind: PlanKind,
    pub subtype: Option<String>,
    /// The repository a software plan names (ADR 0086).
    pub repository: Option<String>,
    /// How an operations plan's service takes requests (ADR 0059), if it does.
    pub requests: Option<String>,
    pub yes: u32,
    pub no: u32,
    /// When its Outcome closed it, and who closed it.
    pub at: Timestamp,
    pub closed_by: AgentPubKey,
    pub score: Score,
}

/// Approved plans of kinds this project's mode takes, in the order they closed.
pub fn approved(
    decisions: &[Decision],
    mode: &str,
    standing: &mut Standing,
) -> ExternResult<Vec<Approved>> {
    let mut out = Vec::new();
    for d in decisions {
        let ProposalKind::Plan(plan) = &d.proposal.entry.kind else {
            continue;
        };
        if !plan.kind.allowed(mode) {
            continue;
        }
        let ratings = rated(&d.proposal.hash, d.proposal.at, standing)?;
        out.push(Approved {
            hash: d.proposal.hash.clone(),
            author: d.proposal.author.clone(),
            kind: plan.kind,
            subtype: plan.subtype.clone(),
            repository: plan.repository_url.clone(),
            requests: plan.requests.clone(),
            yes: d.yes,
            no: d.no,
            at: d.at,
            closed_by: d.closed_by.clone(),
            score: score(&ratings),
        });
    }
    Ok(out)
}

/// The kinds whose plans compete with a `kind` plan for the lead in a space founded as
/// `born`: just `kind` in a typed project, every type-giving kind in a proposal (ADR 0059).
pub fn rivals(born: &str, kind: PlanKind) -> Vec<PlanKind> {
    let open = PlanKind::open_kinds(born, kind.phase());
    if open.contains(&kind) {
        open
    } else {
        vec![kind]
    }
}

/// The leading approved plan among `kinds` (one kind, or a proposal's competing kinds).
pub fn leader(approved: &[Approved], kinds: &[PlanKind]) -> Leader<ActionHash> {
    // Organizers set the plan (ADR 0061): their newest approved plan leads.
    if project_integrity::organized().unwrap_or(false) {
        return approved
            .iter()
            .rev()
            .find(|a| kinds.contains(&a.kind))
            .map_or(Leader::None, |a| Leader::One(a.hash.clone()));
    }
    let candidates: Vec<Candidate<ActionHash>> = approved
        .iter()
        .filter(|a| kinds.contains(&a.kind))
        .map(|a| Candidate {
            id: a.hash.clone(),
            yes: a.yes,
            no: a.no,
            score: a.score,
        })
        .collect();
    pick_leader(&candidates)
}

/// The plan leading the first plan phase of a space founded as `born`, if exactly one does.
pub fn first_leader<'a>(approved: &'a [Approved], born: &str) -> Option<&'a Approved> {
    let groups: Vec<Vec<PlanKind>> = if born == PROPOSAL {
        vec![PlanKind::open_kinds(PROPOSAL, Phase::P2)]
    } else {
        vec![vec![PlanKind::Production], vec![PlanKind::Organisation]]
    };
    groups
        .iter()
        .find_map(|kinds| match leader(approved, kinds) {
            Leader::One(hash) => approved.iter().find(|a| a.hash == hash),
            _ => None,
        })
}

/// The subtype of the leading production or operations plan, if there is one.
pub fn live_subtype(approved: &[Approved], born: &str) -> Option<String> {
    first_leader(approved, born).and_then(|a| a.subtype.clone())
}

/// Every rated criterion's ratings (rubric or value; integrity checked the criterion) from
/// members with Full standing before the plan, latest rating per member (ADR 0039, 0041).
pub fn rated(
    proposal: &ActionHash,
    proposed_at: Timestamp,
    standing: &mut Standing,
) -> ExternResult<BTreeMap<String, Ratings>> {
    let mut values: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for r in latest_ratings(proposal)? {
        if let Some(value) = r.entry.rating {
            if standing.full(&r.author, proposed_at)? {
                values.entry(r.entry.criterion).or_default().push(value);
            }
        }
    }
    Ok(values
        .into_iter()
        .map(|(id, v)| (id, summarize(&v)))
        .collect())
}

/// web's `plan_average_rating`: the mean over every rated criterion.
pub fn score(rated: &BTreeMap<String, Ratings>) -> Score {
    Score::of(&rated.values().copied().collect::<Vec<_>>())
}

/// Each member's latest rating per criterion, from the network and the caller's own chain.
pub fn latest_ratings(proposal: &ActionHash) -> ExternResult<Vec<Found<Rating>>> {
    let mut latest: BTreeMap<(AgentPubKey, String), Found<Rating>> = BTreeMap::new();
    for r in reads::ratings(proposal)? {
        let key = (r.author.clone(), r.entry.criterion.clone());
        if latest.get(&key).is_none_or(|old| r.at > old.at) {
            latest.insert(key, r);
        }
    }
    Ok(latest.into_values().collect())
}
