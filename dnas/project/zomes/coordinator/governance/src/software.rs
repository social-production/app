//! Software projects (web-backend's `services/projects/software`, ADR 0086): the software
//! proposals and their decisions, with every time a production or operations plan took the
//! lead, replayed in time order (`sp_software::replay`); and the checks before proposing one.

use hdk::prelude::*;
use project_integrity::{Proposal, ProposalKind, SoftwareRequest};
use sp_governance::plans::PlanKind;
use sp_software::{replay, Event, Software, Stage};

use crate::apply::{self, Standing, State};
use crate::count::count;
use crate::plan_lead;
use crate::reads::{self, Found};
use crate::roll::Roll;
use crate::write::guest;

pub type Replayed = Software<ActionHash, AgentPubKey>;

/// A software proposal and how it was decided: `Some(true)` passed (with the recount),
/// `Some(false)` rejected, `None` open.
pub struct Request {
    pub found: Found<Proposal>,
    pub kind: SoftwareRequest,
    pub decided: Option<(bool, Timestamp)>,
}

pub struct Ledger {
    pub state: State,
    pub software: Replayed,
    /// Every software proposal, oldest first.
    pub requests: Vec<Request>,
}

pub fn ledger(roll: &Roll) -> ExternResult<Ledger> {
    let state = apply::state(&mut Standing::default(), roll)?;
    let mut requests = Vec::new();
    for found in reads::proposals()? {
        let ProposalKind::Software(request) = found.entry.kind.clone() else {
            continue;
        };
        let passed = state.software.iter().find(|(h, _)| h == &found.hash);
        let decided = match passed {
            Some((_, at)) => Some((true, *at)),
            // A passed Outcome whose recount fails didn't pass either.
            None => count(&found, roll)?.decided.map(|o| (false, o.at)),
        };
        requests.push(Request {
            found,
            kind: request,
            decided,
        });
    }
    let software = replay(events(&state, &requests));
    Ok(Ledger {
        state,
        software,
        requests,
    })
}

fn events(state: &State, requests: &[Request]) -> Vec<Event<ActionHash, AgentPubKey>> {
    let mut timed = Vec::new();
    for lead in plan_lead::replay(&state.approved) {
        if !matches!(lead.kind, PlanKind::Production | PlanKind::Organisation) {
            continue;
        }
        let plan = state.approved.iter().find(|a| a.hash == lead.plan);
        let repository = plan.and_then(|a| a.repository.clone()).unwrap_or_default();
        timed.push((
            lead.at,
            Event::Lead {
                author: lead.author,
                repository,
            },
        ));
    }
    for r in requests {
        let hash = r.found.hash.clone();
        if let Some(filed) = filed(r) {
            timed.push((r.found.at, filed));
        }
        let Some((passed, at)) = r.decided else {
            continue;
        };
        let decision = match &r.kind {
            SoftwareRequest::PullRequest { .. } => Some(Event::PullRequestDecided {
                proposal: hash,
                passed,
            }),
            SoftwareRequest::Merge { .. } => Some(Event::MergeDecided {
                proposal: hash,
                passed,
            }),
            SoftwareRequest::MergeCapability { agent, grant } => {
                passed.then(|| Event::Capability {
                    target: agent.clone(),
                    grant: *grant,
                })
            }
            SoftwareRequest::Repository {
                repository_url,
                pull_request,
                ..
            } => passed.then(|| Event::Repository {
                proposal: hash,
                pull_request: pull_request.clone(),
                author: r.found.author.clone(),
                url: repository_url.clone(),
            }),
        };
        timed.extend(decision.map(|d| (at, d)));
    }
    timed.sort_by_key(|(at, _)| *at);
    timed.into_iter().map(|(_, e)| e).collect()
}

/// What filing `r` does by itself: a pull request opens, a merge is recorded.
fn filed(r: &Request) -> Option<Event<ActionHash, AgentPubKey>> {
    match &r.kind {
        SoftwareRequest::PullRequest { .. } => Some(Event::PullRequest {
            proposal: r.found.hash.clone(),
        }),
        SoftwareRequest::Merge {
            pull_request,
            merge_id,
            merge_url,
        } => Some(Event::Merge {
            proposal: r.found.hash.clone(),
            pull_request: pull_request.clone(),
            author: r.found.author.clone(),
            id: merge_id.clone(),
            url: merge_url.clone(),
        }),
        _ => None,
    }
}

/// web-backend's refusals before a software proposal is filed.
pub fn check(request: &SoftwareRequest) -> ExternResult<()> {
    let roll = Roll::load()?;
    let ledger = ledger(&roll)?;
    if !sp_software::takes_software(ledger.state.mode())
        || ledger.state.subtype() != sp_software::SUBTYPE
    {
        return Err(guest("Software governance requires a software project"));
    }
    let me = agent_info()?.agent_initial_pubkey;
    let stage = |pr: &ActionHash| ledger.software.pull_request(pr).map(|p| p.stage);
    match request {
        SoftwareRequest::PullRequest { .. } => Ok(()),
        SoftwareRequest::Merge { pull_request, .. } => {
            if !ledger.software.can_merge(&me) {
                return Err(guest(
                    "Only members with merge capability can record merges",
                ));
            }
            if stage(pull_request) != Some(Stage::AwaitingMerge) {
                return Err(guest(
                    "Pull request must be approved before merge recording",
                ));
            }
            Ok(())
        }
        SoftwareRequest::MergeCapability { agent, .. } => {
            if !roll.members_at(sys_time()?).contains_key(agent) {
                return Err(guest("Only project members can perform this action"));
            }
            Ok(())
        }
        SoftwareRequest::Repository { pull_request, .. } => match stage(pull_request) {
            Some(_) => Ok(()),
            None => Err(guest("Related pull request not found")),
        },
    }
}
