//! M3c phase returns: members propose going back to an earlier planning phase, with a reason
//! and without the proposer's automatic Yes, as on web (`revert.py`, ADR 0040).

use holochain::prelude::*;
use serde::{Deserialize, Serialize};

mod common;

use common::net::Net;

/// Mirrors the moves of `project_integrity::ProposalKind` (tests can't link two integrity
/// crates).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum ProposalKind {
    Phase {
        to: String,
        plan: Option<ActionHash>,
        reason: String,
    },
    Return {
        to: String,
        reason: String,
    },
}

#[derive(Serialize, Debug)]
struct VoteInput {
    proposal: ActionHash,
    choice: &'static str,
}

#[derive(Deserialize, Debug)]
struct Step {
    id: String,
}

#[derive(Deserialize, Debug)]
struct ProjectView {
    phase: String,
    stage: String,
    next_phase: Option<Step>,
    returns: Vec<Step>,
}

#[derive(Deserialize, Debug)]
struct Listed {
    id: ActionHash,
    yes: u32,
    status: String,
}

fn move_to(to: &str) -> ProposalKind {
    ProposalKind::Phase {
        to: to.into(),
        plan: None,
        reason: "The plan is ready.".into(),
    }
}

fn back_to(to: &str, reason: &str) -> ProposalKind {
    ProposalKind::Return {
        to: to.into(),
        reason: reason.into(),
    }
}

async fn vote_yes(net: &Net, i: usize, proposal: &ActionHash) {
    let input = VoteInput {
        proposal: proposal.clone(),
        choice: "Yes",
    };
    let _: ActionHash = net.call(i, "governance", "vote", input).await;
}

async fn project(net: &Net, i: usize) -> ProjectView {
    let view: Option<ProjectView> = net.call(i, "project", "get_project", ()).await;
    view.expect("project view")
}

/// The proposer moves on; the other founder's Yes carries it.
async fn advance(net: &Net, to: &str) {
    let id: ActionHash = net.call(0, "governance", "propose", move_to(to)).await;
    net.settle().await;
    vote_yes(net, 1, &id).await;
    net.settle().await;
    assert_eq!(project(net, 0).await.phase, to);
}

async fn refused(net: &Net, kind: ProposalKind, why: &str) {
    let result = net
        .try_call::<_, ActionHash>(0, "governance", "propose", kind)
        .await;
    assert!(result.is_err(), "{why}");
}

#[tokio::test(flavor = "multi_thread")]
async fn members_can_return_a_project_to_an_earlier_phase() {
    let net = Net::new(2, 2, "returns").await;
    net.demand_all().await;
    assert!(
        project(&net, 0).await.returns.is_empty(),
        "nothing before Proposal"
    );
    advance(&net, "phase-2").await;
    let view = project(&net, 1).await;
    let offered: Vec<&str> = view.returns.iter().map(|s| s.id.as_str()).collect();
    assert_eq!(offered, ["phase-1"], "web's revertablePhaseIds");

    refused(&net, back_to("phase-2", "Again."), "not the current phase").await;
    refused(&net, back_to("phase-3", "Ahead."), "not forward").await;
    refused(&net, back_to("phase-1", "   "), "a reason is required").await;

    let id: ActionHash = net
        .call(
            0,
            "governance",
            "propose",
            back_to("phase-1", "Support dropped."),
        )
        .await;
    net.settle().await;
    let listed: Vec<Listed> = net.call(1, "governance", "list_proposals", ()).await;
    let open = listed.iter().find(|p| p.id == id).expect("listed");
    assert_eq!(
        (open.yes, open.status.as_str()),
        (0, "Open"),
        "a return comes without its proposer's Yes, as on web"
    );
    refused(
        &net,
        back_to("phase-1", "Twice."),
        "one phase vote at a time",
    )
    .await;

    vote_yes(&net, 0, &id).await;
    net.settle().await;
    vote_yes(&net, 1, &id).await;
    net.settle().await;
    let view = project(&net, 1).await;
    assert_eq!(
        (view.phase.as_str(), view.stage.as_str()),
        ("phase-1", "Proposal")
    );
    assert!(view.returns.is_empty());
    assert_eq!(view.next_phase.map(|s| s.id).as_deref(), Some("phase-2"));

    // The fold carries on from the phase the return reached.
    advance(&net, "phase-2").await;
}
