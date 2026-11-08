//! Software projects (ADR 0086): the leading software plan's author may merge; a pull request
//! is voted, its merge recorded by a merger and confirmed by a second vote; members grant
//! merge capability and replace the repository, which hands merging to the replacement's
//! author.

use holochain::prelude::*;
use serde::{Deserialize, Serialize};
use sp_governance::plans::PlanKind;

mod common;

use common::net::Net;
use common::plans::{Plan, PlanBoard, PlanStage, RateInput, VoteInput};

const OLD_REPO: &str = "https://example.org/garden.git";
const NEW_REPO: &str = "https://example.org/garden-new.git";

#[derive(Serialize, Debug, Clone)]
enum SoftwareAsk {
    PullRequest {
        title: String,
        summary: String,
        pull_request_id: String,
        pull_request_url: String,
    },
    Merge {
        pull_request: ActionHash,
        merge_id: String,
        merge_url: String,
    },
    MergeCapability {
        agent: AgentPubKey,
        grant: bool,
    },
    Repository {
        repository_url: String,
        reason: String,
        pull_request: ActionHash,
    },
}

#[derive(Serialize, Debug, Clone)]
#[allow(clippy::large_enum_variant)]
enum Kind {
    Phase {
        to: String,
        plan: Option<ActionHash>,
        reason: String,
    },
    Plan(Plan),
    Software(SoftwareAsk),
}

#[derive(Deserialize, Debug, PartialEq)]
struct Merger {
    agent: AgentPubKey,
    source: String,
}

#[derive(Deserialize, Debug)]
struct Tally {
    yes: u32,
    status: String,
}

#[derive(Deserialize, Debug)]
struct MergeView {
    proposal: ActionHash,
    by: AgentPubKey,
}

#[derive(Deserialize, Debug)]
struct PullRequestView {
    id: ActionHash,
    stage: String,
    merge: Option<MergeView>,
    tally: Tally,
    can_record_merge: bool,
}

#[derive(Deserialize, Debug)]
struct RepositoryView {
    previous_repository_url: String,
    tally: Tally,
}

#[derive(Deserialize, Debug)]
struct SoftwareView {
    repository_url: String,
    mergers: Vec<Merger>,
    candidates: Vec<AgentPubKey>,
    pull_requests: Vec<PullRequestView>,
    repository_requests: Vec<RepositoryView>,
    can_propose: bool,
}

fn agent(net: &Net, i: usize) -> AgentPubKey {
    net.cells[i].1.agent_pubkey().clone()
}

fn merger(net: &Net, i: usize, source: &str) -> Merger {
    Merger {
        agent: agent(net, i),
        source: source.into(),
    }
}

async fn propose(net: &Net, i: usize, kind: Kind) -> ActionHash {
    let id = net.call(i, "governance", "propose", kind).await;
    net.settle().await;
    id
}

async fn both_vote_yes(net: &Net, proposal: &ActionHash) {
    for i in 0..2 {
        let input = VoteInput {
            proposal: proposal.clone(),
            choice: "Yes",
        };
        let _: ActionHash = net.call(i, "governance", "vote", input).await;
        net.settle().await;
    }
}

async fn view(net: &Net, i: usize) -> SoftwareView {
    net.call(i, "governance", "software", ()).await
}

async fn pull_request(net: &Net, i: usize, id: &ActionHash) -> PullRequestView {
    let mut all = view(net, i).await.pull_requests;
    let at = all.iter().position(|p| &p.id == id).expect("listed");
    all.swap_remove(at)
}

fn pr(title: &str) -> Kind {
    Kind::Software(SoftwareAsk::PullRequest {
        title: title.into(),
        summary: "Waters the beds on a timer.".into(),
        pull_request_id: "#12".into(),
        pull_request_url: "https://example.org/garden/pull/12".into(),
    })
}

fn merge(pull_request: &ActionHash) -> Kind {
    Kind::Software(SoftwareAsk::Merge {
        pull_request: pull_request.clone(),
        merge_id: "a1b2c3".into(),
        merge_url: "https://example.org/garden/commit/a1b2c3".into(),
    })
}

/// Two founders reach Production Plan and approve agent 1's software plan, which leads.
async fn software_project() -> Net {
    let net = Net::new(2, 2, "software").await;
    net.demand_all().await;
    let to_plans = Kind::Phase {
        to: "phase-2".into(),
        plan: None,
        reason: "Ready to plan.".into(),
    };
    let id = propose(&net, 0, to_plans).await;
    let vote = VoteInput {
        proposal: id,
        choice: "Yes",
    };
    let _: ActionHash = net.call(1, "governance", "vote", vote).await;
    net.settle().await;
    let plan = Kind::Plan(Plan {
        kind: PlanKind::Production,
        title: "Watering app".into(),
        description: "A small app that waters the beds.".into(),
        demand_note: "Everyone who asked.".into(),
        subtype: Some("software".into()),
        repository_url: Some(OLD_REPO.into()),
        location: None,
        recipients: None,
        stages: vec![PlanStage {
            title: "Build".into(),
            details: "Write it.".into(),
            start_date: Some("2026-11-10".into()),
            end_date: None,
        }],
        schedule: None,
    });
    let plan = propose(&net, 1, plan).await;
    let board: PlanBoard = net.call(0, "governance", "list_plans", ()).await;
    for i in 0..2 {
        for c in &board.plans[0].criteria {
            let input = RateInput {
                proposal: plan.clone(),
                criterion: c.id.clone(),
                rating: Some(4),
            };
            let _: ActionHash = net.call(i, "governance", "rate", input).await;
        }
    }
    both_vote_yes(&net, &plan).await;
    net
}

/// Vote, record (only a merger, only once approved), confirm.
async fn merged(net: &Net) {
    let id = propose(net, 0, pr("Timer")).await;
    let opened = pull_request(net, 0, &id).await;
    assert_eq!((opened.stage.as_str(), opened.tally.yes), ("approval", 0));
    let early = net
        .try_call::<_, ActionHash>(1, "governance", "propose", merge(&id))
        .await;
    assert!(early.is_err(), "not approved yet");
    both_vote_yes(net, &id).await;
    assert_eq!(pull_request(net, 0, &id).await.stage, "awaiting-merge");
    assert!(!pull_request(net, 0, &id).await.can_record_merge);
    assert!(pull_request(net, 1, &id).await.can_record_merge);
    let refused = net
        .try_call::<_, ActionHash>(0, "governance", "propose", merge(&id))
        .await;
    assert!(refused.is_err(), "agent 0 can't merge");

    let recorded = propose(net, 1, merge(&id)).await;
    let waiting = pull_request(net, 0, &id).await;
    assert_eq!(waiting.stage, "confirmation");
    let m = waiting.merge.expect("recorded");
    assert_eq!((m.proposal, m.by), (recorded.clone(), agent(net, 1)));
    assert_eq!(waiting.tally.yes, 0, "the confirmation vote starts empty");
    both_vote_yes(net, &recorded).await;
    assert_eq!(pull_request(net, 0, &id).await.stage, "confirmed");
}

#[tokio::test(flavor = "multi_thread")]
async fn pull_requests_merges_and_the_repository() {
    let net = software_project().await;
    let start = view(&net, 0).await;
    assert_eq!(start.repository_url, OLD_REPO);
    assert_eq!(start.mergers, vec![merger(&net, 1, "plan-creator")]);
    assert_eq!(start.candidates, vec![agent(&net, 0)]);
    assert!(start.can_propose);

    merged(&net).await;

    let grant = Kind::Software(SoftwareAsk::MergeCapability {
        agent: agent(&net, 0),
        grant: true,
    });
    let grant = propose(&net, 1, grant).await;
    both_vote_yes(&net, &grant).await;
    assert!(view(&net, 1)
        .await
        .mergers
        .contains(&merger(&net, 0, "approved-request")));

    let blocked = propose(&net, 0, pr("Sensors")).await;
    both_vote_yes(&net, &blocked).await;
    let replace = Kind::Software(SoftwareAsk::Repository {
        repository_url: NEW_REPO.into(),
        reason: "The old host closed.".into(),
        pull_request: blocked.clone(),
    });
    let replace = propose(&net, 0, replace).await;
    both_vote_yes(&net, &replace).await;
    let after = view(&net, 1).await;
    assert_eq!(after.repository_url, NEW_REPO);
    assert_eq!(pull_request(&net, 1, &blocked).await.stage, "replaced");
    assert_eq!(after.mergers, vec![merger(&net, 0, "approved-request")]);
    let record = &after.repository_requests[0];
    assert_eq!(
        (
            record.previous_repository_url.as_str(),
            record.tally.status.as_str()
        ),
        (OLD_REPO, "Passed")
    );
    net.shut_down().await;
}
