//! What web's software panel shows (`ProjectSoftwareGovernanceDataOut`, ADR 0086): the
//! repository and who may merge, every pull request with its stage, the merge capability and
//! repository requests, and the replacements that passed.

use hdk::prelude::*;
use project_integrity::SoftwareRequest;
use sp_software::{Merger, Stage};

use crate::calls;
use crate::roll::Roll;
use crate::software::{ledger, Ledger, Request};
use crate::views::{tally_view, TallyView};

#[derive(Serialize, Deserialize, Debug)]
pub struct MergeView {
    /// The proposal members confirm the merge with.
    pub proposal: ActionHash,
    pub merge_id: String,
    pub merge_url: String,
    pub by: AgentPubKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PullRequestView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub created_at: Timestamp,
    pub title: String,
    pub summary: String,
    pub pull_request_id: String,
    pub pull_request_url: String,
    pub stage: Stage,
    pub stage_label: String,
    pub merge: Option<MergeView>,
    /// The vote now open or last held: the merge's while it is confirmed, else the pull
    /// request's own.
    pub tally: TallyView,
    pub can_record_merge: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CapabilityView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub created_at: Timestamp,
    pub agent: AgentPubKey,
    pub grant: bool,
    pub tally: TallyView,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RepositoryView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub created_at: Timestamp,
    pub repository_url: String,
    /// The repository it replaced (or would replace, while open).
    pub previous_repository_url: String,
    pub reason: String,
    pub pull_request: ActionHash,
    pub tally: TallyView,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SoftwareView {
    pub repository_url: String,
    pub mergers: Vec<Merger<AgentPubKey>>,
    /// Members who can't merge (web's `availableMergeCapabilityCandidates`).
    pub candidates: Vec<AgentPubKey>,
    /// Newest first, as web lists them.
    pub pull_requests: Vec<PullRequestView>,
    pub capability_requests: Vec<CapabilityView>,
    pub repository_requests: Vec<RepositoryView>,
    /// Members file pull requests, merge capability changes and replacements.
    pub can_propose: bool,
}

/// What every request's view needs.
struct Ctx<'a> {
    ledger: &'a Ledger,
    me: AgentPubKey,
    roll: &'a Roll,
    member: bool,
}

pub fn view() -> ExternResult<SoftwareView> {
    let roll = Roll::load()?;
    let ledger = ledger(&roll)?;
    let sw = &ledger.software;
    let candidates = roll
        .members_at(sys_time()?)
        .into_keys()
        .filter(|a| !sw.can_merge(a))
        .collect();
    let ctx = Ctx {
        ledger: &ledger,
        me: agent_info()?.agent_initial_pubkey,
        roll: &roll,
        member: calls::my_membership()?.is_some(),
    };
    let mut out = SoftwareView {
        repository_url: sw.repository.clone(),
        mergers: sw.mergers.clone(),
        candidates,
        pull_requests: Vec::new(),
        capability_requests: Vec::new(),
        repository_requests: Vec::new(),
        can_propose: ctx.member,
    };
    for r in ledger.requests.iter().rev() {
        add(&mut out, &ctx, r)?;
    }
    Ok(out)
}

fn add(out: &mut SoftwareView, ctx: &Ctx, r: &Request) -> ExternResult<()> {
    let f = &r.found;
    match &r.kind {
        SoftwareRequest::PullRequest { .. } => out.pull_requests.extend(pull_request(ctx, r)?),
        // A merge shows on its pull request.
        SoftwareRequest::Merge { .. } => {}
        SoftwareRequest::MergeCapability { agent, grant } => {
            out.capability_requests.push(CapabilityView {
                id: f.hash.clone(),
                author: f.author.clone(),
                created_at: f.at,
                agent: agent.clone(),
                grant: *grant,
                tally: tally_view(f, &ctx.me, ctx.roll)?,
            });
        }
        SoftwareRequest::Repository {
            repository_url,
            reason,
            pull_request,
        } => {
            let sw = &ctx.ledger.software;
            let replaced = sw.replaced.iter().find(|(h, _)| h == &f.hash);
            out.repository_requests.push(RepositoryView {
                id: f.hash.clone(),
                author: f.author.clone(),
                created_at: f.at,
                repository_url: repository_url.clone(),
                previous_repository_url: replaced
                    .map_or_else(|| sw.repository.clone(), |(_, p)| p.clone()),
                reason: reason.clone(),
                pull_request: pull_request.clone(),
                tally: tally_view(f, &ctx.me, ctx.roll)?,
            });
        }
    }
    Ok(())
}

fn pull_request(ctx: &Ctx, r: &Request) -> ExternResult<Option<PullRequestView>> {
    let SoftwareRequest::PullRequest {
        title,
        summary,
        pull_request_id,
        pull_request_url,
    } = &r.kind
    else {
        return Ok(None);
    };
    let f = &r.found;
    let Some(pr) = ctx.ledger.software.pull_request(&f.hash) else {
        return Ok(None);
    };
    let merge = pr.merge.as_ref();
    let confirming = merge.and_then(|m| {
        ctx.ledger
            .requests
            .iter()
            .find(|q| q.found.hash == m.proposal)
    });
    let voted = confirming.map_or(f, |q| &q.found);
    Ok(Some(PullRequestView {
        id: f.hash.clone(),
        author: f.author.clone(),
        created_at: f.at,
        title: title.clone(),
        summary: summary.clone(),
        pull_request_id: pull_request_id.clone(),
        pull_request_url: pull_request_url.clone(),
        stage: pr.stage,
        stage_label: pr.stage.label().into(),
        merge: merge.map(|m| MergeView {
            proposal: m.proposal.clone(),
            merge_id: m.id.clone(),
            merge_url: m.url.clone(),
            by: m.by.clone(),
        }),
        tally: tally_view(voted, &ctx.me, ctx.roll)?,
        can_record_merge: ctx.member
            && pr.stage == Stage::AwaitingMerge
            && ctx.ledger.software.can_merge(&ctx.me),
    }))
}
