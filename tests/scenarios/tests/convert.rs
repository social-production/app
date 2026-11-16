//! Convert (ADR 0106): members vote to close a project and continue its work as a new
//! proposal; the two name each other with lineage links.

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
    Convert {
        reason: String,
    },
}

#[derive(Serialize, Debug)]
struct VoteInput {
    proposal: ActionHash,
    choice: &'static str,
}

#[derive(Deserialize, Debug, Default)]
struct Lineage {
    converted: bool,
    continues: Option<String>,
    continued_by: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ProjectView {
    phase: String,
    lineage: Lineage,
}

#[derive(Serialize, Debug)]
struct LineageInput {
    other: String,
    continued_by: bool,
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

fn convert(reason: &str) -> ProposalKind {
    ProposalKind::Convert {
        reason: reason.into(),
    }
}

/// Founder one moves it to its Plan phase; founder two's Yes carries it.
async fn advance(net: &Net) {
    let to_plan = ProposalKind::Phase {
        to: "phase-2".into(),
        plan: None,
        reason: "Ready to plan.".into(),
    };
    let id: ActionHash = net.call(0, "governance", "propose", to_plan).await;
    net.settle().await;
    vote_yes(net, 1, &id).await;
    net.settle().await;
    assert_eq!(project(net, 0).await.phase, "phase-2");
}

#[tokio::test(flavor = "multi_thread")]
async fn members_convert_a_project_into_a_new_proposal() {
    let net = Net::new(2, 2, "convert").await;
    net.demand_all().await;
    let early = net
        .try_call::<_, ActionHash>(0, "governance", "propose", convert("Too soon."))
        .await;
    assert!(early.is_err(), "not from the Proposal phase");

    advance(&net).await;

    let blank = net
        .try_call::<_, ActionHash>(0, "governance", "propose", convert("  "))
        .await;
    assert!(blank.is_err(), "a reason is required");
    let id: ActionHash = net
        .call(0, "governance", "propose", convert("It is a service now."))
        .await;
    net.settle().await;
    assert!(
        !project(&net, 1).await.lineage.converted,
        "not until it passes"
    );
    vote_yes(&net, 1, &id).await;
    net.settle().await;
    let view = project(&net, 1).await;
    assert_eq!(view.phase, "phase-7", "converting closes it");
    assert!(view.lineage.converted);

    let input = LineageInput {
        other: "successor-id".into(),
        continued_by: true,
    };
    let _: ActionHash = net.call(1, "project", "link_lineage", input).await;
    net.settle().await;
    let view = project(&net, 0).await;
    assert_eq!(view.lineage.continued_by.as_deref(), Some("successor-id"));
    assert_eq!(view.lineage.continues, None);
    net.shut_down().await;
}
