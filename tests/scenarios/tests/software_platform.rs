//! Software on a #platform project (ADR 0095, amended 2026-11-18): only platform moderators
//! record merges. The leading plan's author can't, merge capability can't be granted, and a
//! moderator's merge counts whoever they are in the project.

use holochain::prelude::*;
use serde::{Deserialize, Serialize};
use sp_governance::plans::PlanKind;

mod common;

use common::net::{charter, Charter, Net};
use common::plans::{Plan, PlanBoard, PlanStage, RateInput, VoteInput};

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

#[derive(Deserialize, Debug)]
struct MergeView {
    by: AgentPubKey,
}

#[derive(Deserialize, Debug)]
struct PullRequestView {
    id: ActionHash,
    stage: String,
    merge: Option<MergeView>,
    can_record_merge: bool,
}

#[derive(Deserialize, Debug)]
struct Merger {
    agent: AgentPubKey,
}

#[derive(Deserialize, Debug)]
struct SoftwareView {
    platform: bool,
    mergers: Vec<Merger>,
    pull_requests: Vec<PullRequestView>,
}

const MEMBERS: usize = 3;

fn agent(net: &Net, i: usize) -> AgentPubKey {
    net.cells[i].1.agent_pubkey().clone()
}

async fn propose(net: &Net, i: usize, kind: Kind) -> ActionHash {
    let id = net.call(i, "governance", "propose", kind).await;
    net.settle().await;
    id
}

/// Everyone votes Yes. A proposer's phase vote is already cast, and two of three Yes votes
/// can close a vote before the third, so refusals are skipped; the stages checked after show
/// each vote passed.
async fn all_vote_yes(net: &Net, proposal: &ActionHash) {
    for i in 0..MEMBERS {
        let input = VoteInput {
            proposal: proposal.clone(),
            choice: "Yes",
        };
        let voted = net
            .try_call::<_, ActionHash>(i, "governance", "vote", input)
            .await;
        if let Err(e) = voted {
            eprintln!("agent {i}'s vote skipped: {e}");
        }
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

fn merge(pull_request: &ActionHash) -> Kind {
    Kind::Software(SoftwareAsk::Merge {
        pull_request: pull_request.clone(),
        merge_id: "a1b2c3".into(),
        merge_url: "https://example.org/net/commit/a1b2c3".into(),
    })
}

/// Agent 2 stands for the board; nobody was active last week, so they're seated at once.
async fn seat_moderator(net: &Net) {
    let zome = net.cells[2].0.zome("identity");
    let _: ActionHash = net.conductors[2].call(&zome, "volunteer_board", true).await;
    net.settle().await;
    let zome = net.cells[0].0.zome("identity");
    let seated: bool = net.conductors[0]
        .call(&zome, "is_moderator", agent(net, 2))
        .await;
    assert!(seated, "agent 2 is a moderator");
}

/// Three Full founders reach Production Plan and approve agent 1's software plan.
async fn platform_software() -> Net {
    let platform = Charter {
        tags: vec!["#platform".into()],
        ..charter("productive")
    };
    let net = Net::with_charter(MEMBERS, MEMBERS, "platform-software", platform).await;
    net.demand_all().await;
    let to_plans = Kind::Phase {
        to: "phase-2".into(),
        plan: None,
        reason: "Ready to plan.".into(),
    };
    let id = propose(&net, 0, to_plans).await;
    all_vote_yes(&net, &id).await;
    let plan = Kind::Plan(Plan {
        kind: PlanKind::Production,
        title: "Network upgrade".into(),
        description: "The next release of the app.".into(),
        demand_note: "Everyone on the network.".into(),
        subtype: Some("software".into()),
        repository_url: Some("https://example.org/net.git".into()),
        location: None,
        recipients: None,
        stages: vec![PlanStage {
            title: "Build".into(),
            details: "Write it.".into(),
            start_date: Some("2026-11-10".into()),
            end_date: None,
        }],
        schedule: None,
        requests: None,
    });
    let plan = propose(&net, 1, plan).await;
    let board: PlanBoard = net.call(0, "governance", "list_plans", ()).await;
    for i in 0..MEMBERS {
        for c in &board.plans[0].criteria {
            let input = RateInput {
                proposal: plan.clone(),
                criterion: c.id.clone(),
                rating: Some(4),
            };
            let _: ActionHash = net.call(i, "governance", "rate", input).await;
        }
    }
    all_vote_yes(&net, &plan).await;
    net
}

#[tokio::test(flavor = "multi_thread")]
async fn only_platform_moderators_merge_on_platform_projects() {
    let net = platform_software().await;
    seat_moderator(&net).await;
    let start = view(&net, 0).await;
    assert!(start.platform);
    assert!(
        start.mergers.is_empty(),
        "the plan's author merges nowhere on #platform"
    );

    let pr = Kind::Software(SoftwareAsk::PullRequest {
        title: "Release 1.1".into(),
        summary: "Fixes and a faster feed.".into(),
        pull_request_id: "#7".into(),
        pull_request_url: "https://example.org/net/pull/7".into(),
    });
    let id = propose(&net, 0, pr).await;
    all_vote_yes(&net, &id).await;
    assert_eq!(pull_request(&net, 0, &id).await.stage, "awaiting-merge");
    assert!(!pull_request(&net, 1, &id).await.can_record_merge);
    assert!(pull_request(&net, 2, &id).await.can_record_merge);
    let author = net
        .try_call::<_, ActionHash>(1, "governance", "propose", merge(&id))
        .await;
    assert!(author.is_err(), "the plan's author can't merge: {author:?}");

    let _ = propose(&net, 2, merge(&id)).await;
    let waiting = pull_request(&net, 0, &id).await;
    assert_eq!(waiting.stage, "confirmation");
    assert_eq!(waiting.merge.map(|m| m.by), Some(agent(&net, 2)));

    let grant = Kind::Software(SoftwareAsk::MergeCapability {
        agent: agent(&net, 0),
        grant: true,
    });
    let grant = net
        .try_call::<_, ActionHash>(1, "governance", "propose", grant)
        .await;
    assert!(grant.is_err(), "no capability on #platform: {grant:?}");
    net.shut_down().await;
}
