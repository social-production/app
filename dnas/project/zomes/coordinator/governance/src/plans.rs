//! Plan selection (ADR 0039): approved plans are passed `Plan` proposals; the leader of each
//! kind is picked from them by approval ratio, then by members' average rating
//! (`sp_governance::plans::pick_leader`, web's `sync_project_plan_leading_flags`).

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::{Plan, ProposalKind, Rating};
use sp_governance::plans::{
    criteria, pick_leader, summarize, Candidate, Leader, PlanKind, Ratings, Score,
};

use crate::decide::{Decision, Standing};
use crate::reads::{self, Found};

/// A plan members approved (its recount passed).
pub struct Approved {
    pub hash: ActionHash,
    pub kind: PlanKind,
    pub subtype: Option<String>,
    pub yes: u32,
    pub no: u32,
    /// When its Outcome closed it.
    pub at: Timestamp,
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
        let ratings = rated(&d.proposal.hash, d.proposal.at, plan, standing)?;
        out.push(Approved {
            hash: d.proposal.hash.clone(),
            kind: plan.kind,
            subtype: plan.subtype.clone(),
            yes: d.yes,
            no: d.no,
            at: d.at,
            score: Score::of(&ratings.iter().map(|(_, r)| *r).collect::<Vec<_>>()),
        });
    }
    Ok(out)
}

pub fn leader(approved: &[Approved], kind: PlanKind) -> Leader<ActionHash> {
    let candidates: Vec<Candidate<ActionHash>> = approved
        .iter()
        .filter(|a| a.kind == kind)
        .map(|a| Candidate {
            id: a.hash.clone(),
            yes: a.yes,
            no: a.no,
            score: a.score,
        })
        .collect();
    pick_leader(&candidates)
}

/// The subtype of the leading production or operations plan, if there is one.
pub fn live_subtype(approved: &[Approved]) -> Option<String> {
    let kinds = [PlanKind::Production, PlanKind::Organisation];
    kinds
        .into_iter()
        .find_map(|kind| match leader(approved, kind) {
            Leader::One(hash) => approved
                .iter()
                .find(|a| a.hash == hash)
                .and_then(|a| a.subtype.clone()),
            _ => None,
        })
}

/// Each rubric criterion's ratings from members with Full standing before the plan, latest
/// rating per member (ADR 0039).
pub fn rated(
    proposal: &ActionHash,
    proposed_at: Timestamp,
    plan: &Plan,
    standing: &mut Standing,
) -> ExternResult<Vec<(&'static str, Ratings)>> {
    let mut values: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    for r in latest_ratings(proposal)? {
        if let Some(value) = r.entry.rating {
            if standing.full(&r.author, proposed_at)? {
                values.entry(r.entry.criterion).or_default().push(value);
            }
        }
    }
    let rubric = criteria(plan.kind, plan.subtype_or_standard());
    Ok(rubric
        .into_iter()
        .map(|c| {
            (
                c.id,
                summarize(values.get(c.id).map_or(&[][..], Vec::as_slice)),
            )
        })
        .collect())
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
