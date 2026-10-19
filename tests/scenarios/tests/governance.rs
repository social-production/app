//! M3 governance: Proposal → Vote → Outcome, applied at read time with standing checks
//! (ADR 0033).

use holochain::prelude::*;
use serde::{Deserialize, Serialize};

mod common;

use common::net::{Charter, Net};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum ProposalKind {
    Edit {
        title: Option<String>,
        description: Option<String>,
        location: Option<String>,
    },
    Update {
        body: String,
    },
    Phase {
        to: String,
        plan: Option<ActionHash>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
enum VoteChoice {
    Yes,
    No,
    Neutral,
}

#[derive(Serialize, Debug)]
struct VoteInput {
    proposal: ActionHash,
    choice: VoteChoice,
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
enum Verdict {
    Passed,
    Rejected,
    Open,
}

#[derive(Deserialize, Debug)]
struct ProposalView {
    id: ActionHash,
    yes: u32,
    no: u32,
    required: u32,
    status: Verdict,
}

#[derive(Deserialize, Debug)]
struct ProjectView {
    charter: Option<Charter>,
    phase: String,
    stage: String,
}

fn rename(title: &str) -> ProposalKind {
    ProposalKind::Edit {
        title: Some(title.into()),
        description: None,
        location: None,
    }
}

async fn vote(
    net: &Net,
    i: usize,
    proposal: &ActionHash,
    choice: VoteChoice,
) -> Result<ActionHash, String> {
    net.try_call(
        i,
        "governance",
        "vote",
        VoteInput {
            proposal: proposal.clone(),
            choice,
        },
    )
    .await
}

async fn project(net: &Net, i: usize) -> ProjectView {
    let view: Option<ProjectView> = net.call(i, "project", "get_project", ()).await;
    view.expect("project view")
}

#[tokio::test(flavor = "multi_thread")]
async fn an_edit_passes_with_enough_yes_votes_and_applies() {
    let net = Net::new(2, 2, "edit").await;
    let id: ActionHash = net
        .call(0, "governance", "propose", rename("Garden co-op"))
        .await;
    net.settle().await;

    let open: Vec<ProposalView> = net.call(1, "governance", "list_proposals", ()).await;
    assert_eq!(
        (open[0].yes, open[0].required, &open[0].status),
        (1, 2, &Verdict::Open),
        "proposer's Yes counts"
    );
    assert!(
        vote(&net, 0, &id, VoteChoice::Yes).await.is_err(),
        "one vote per agent"
    );

    vote(&net, 1, &id, VoteChoice::Yes).await.expect("vote");
    net.settle().await;
    for i in 0..2 {
        let view = project(&net, i).await;
        assert_eq!(view.charter.map(|c| c.title), Some("Garden co-op".into()));
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn rejected_and_ineligible_proposals_change_nothing() {
    // Agents 0 and 1 are founders (Full); agent 2 is New.
    let net = Net::new(3, 2, "reject").await;
    let rejected: ActionHash = net.call(0, "governance", "propose", rename("Nope")).await;
    net.settle().await;
    vote(&net, 1, &rejected, VoteChoice::No)
        .await
        .expect("vote");
    vote(&net, 2, &rejected, VoteChoice::No)
        .await
        .expect("vote");
    net.settle().await;
    // The last voter may not have seen every vote; anyone closes a settled proposal.
    let _: Option<ActionHash> = net
        .call(0, "governance", "close_proposal", rejected.clone())
        .await;
    net.settle().await;
    let list: Vec<ProposalView> = net.call(0, "governance", "list_proposals", ()).await;
    let closed = list.iter().find(|p| p.id == rejected).expect("listed");
    assert_eq!((closed.no, &closed.status), (2, &Verdict::Rejected));

    // Integrity counts the New member's Yes; applying it doesn't (standing filter, ADR 0033).
    let sneaky: ActionHash = net.call(2, "governance", "propose", rename("Sneaky")).await;
    net.settle().await;
    vote(&net, 0, &sneaky, VoteChoice::Yes).await.expect("vote");
    vote(&net, 1, &sneaky, VoteChoice::Yes).await.ok();
    net.settle().await;
    let _: Option<ActionHash> = net
        .call(0, "governance", "close_proposal", sneaky.clone())
        .await;
    net.settle().await;
    // 3 Yes votes close it as passed, but only 2 come from Full members; N = 3 needs 3.
    let list: Vec<ProposalView> = net.call(0, "governance", "list_proposals", ()).await;
    assert_eq!(
        list.iter().find(|p| p.id == sneaky).map(|p| &p.status),
        Some(&Verdict::Passed)
    );
    assert_eq!(
        project(&net, 1).await.charter.map(|c| c.title),
        Some("Garden".into())
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn leaving_proposal_needs_demand_then_a_vote() {
    let net = Net::new(2, 2, "phase").await;
    let to_plan = ProposalKind::Phase {
        to: "phase-2".into(),
        plan: None,
    };
    let early: Result<ActionHash, String> = net
        .try_call(0, "governance", "propose", to_plan.clone())
        .await;
    assert!(early.is_err(), "no demand signals yet");
    let skip: Result<ActionHash, String> = net
        .try_call(
            0,
            "governance",
            "propose",
            ProposalKind::Phase {
                to: "phase-5".into(),
                plan: None,
            },
        )
        .await;
    assert!(skip.is_err(), "phases move one step at a time");

    net.demand_all().await;
    let id: ActionHash = net.call(0, "governance", "propose", to_plan).await;
    net.settle().await;
    vote(&net, 1, &id, VoteChoice::Yes).await.expect("vote");
    net.settle().await;
    let view = project(&net, 0).await;
    assert_eq!(
        (view.phase.as_str(), view.stage.as_str()),
        ("phase-2", "Production Plan")
    );
}
