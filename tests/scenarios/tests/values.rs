//! Project values (ADR 0041): members add values and rate their importance in the Proposal
//! phase; only Full members' latest votes count, and values averaging 5 or more become plan
//! criteria.

use holochain::prelude::*;
use serde::{Deserialize, Serialize};
use sp_governance::plans::PlanKind;

mod common;

use common::net::Net;
use common::plans::{Plan, PlanBoard, PlanStage, ProposalKind, RateInput, VoteInput};

#[derive(Deserialize, Debug)]
struct ValueView {
    id: ActionHash,
    label: String,
    count: u32,
    average_x100: u32,
    mine: Option<u8>,
}

#[derive(Serialize, Debug)]
struct ImportanceInput {
    value: ActionHash,
    importance: u8,
}

async fn add(net: &Net, i: usize, label: &str) -> Result<ActionHash, String> {
    net.try_call(i, "governance", "add_value", label.to_string())
        .await
}

async fn rate(
    net: &Net,
    i: usize,
    value: &ActionHash,
    importance: u8,
) -> Result<ActionHash, String> {
    let input = ImportanceInput {
        value: value.clone(),
        importance,
    };
    net.try_call(i, "governance", "rate_value", input).await
}

async fn values(net: &Net, i: usize) -> Vec<ValueView> {
    net.call(i, "governance", "list_values", ()).await
}

async fn vote_yes(net: &Net, i: usize, proposal: &ActionHash) {
    let input = VoteInput {
        proposal: proposal.clone(),
        choice: "Yes",
    };
    let _: ActionHash = net.call(i, "governance", "vote", input).await;
}

/// Values and their votes: duplicates refused, the latest vote counts, New members' don't.
async fn rated_values(net: &Net) -> ActionHash {
    let space = add(net, 0, "Use unused space").await.expect("added");
    assert!(
        add(net, 1, "  use UNUSED  space ").await.is_err(),
        "already listed"
    );
    assert!(add(net, 1, "   ").await.is_err(), "blank");
    let quiet = add(net, 1, "Quiet at night").await.expect("added");
    net.settle().await;

    rate(net, 0, &space, 8).await.expect("rated");
    rate(net, 1, &space, 2).await.expect("rated");
    rate(net, 1, &space, 6).await.expect("changed");
    rate(net, 2, &space, 1)
        .await
        .expect("a New member may vote");
    rate(net, 0, &quiet, 3).await.expect("rated");
    assert!(rate(net, 0, &quiet, 11).await.is_err(), "1 to 10");
    net.settle().await;

    let seen = values(net, 2).await;
    let labels: Vec<&str> = seen.iter().map(|v| v.label.as_str()).collect();
    assert_eq!(labels, ["Use unused space", "Quiet at night"]);
    let v = &seen[0];
    assert_eq!(
        (v.count, v.average_x100, v.mine),
        (2, 700, Some(1)),
        "Full members' latest votes"
    );
    assert_eq!((seen[1].count, seen[1].average_x100), (1, 300));
    space
}

/// Leaving Proposal closes values; the prominent one becomes a plan criterion.
async fn criteria_on_a_plan(net: &Net, space: &ActionHash) {
    let to_plans = ProposalKind::Phase {
        to: "phase-2".into(),
        plan: None,
        reason: "Values are settled.".into(),
    };
    let id: ActionHash = net.call(0, "governance", "propose", to_plans).await;
    net.settle().await;
    vote_yes(net, 1, &id).await;
    net.settle().await;
    assert!(add(net, 0, "Late").await.is_err(), "Proposal phase only");
    assert!(rate(net, 0, space, 9).await.is_err(), "Proposal phase only");

    let plan: ActionHash = net
        .call(1, "governance", "propose", ProposalKind::Plan(beds()))
        .await;
    net.settle().await;
    let board: PlanBoard = net.call(0, "governance", "list_plans", ()).await;
    let value = board.plans[0].criteria.last().expect("criteria");
    assert_eq!(
        board.plans[0].criteria.len(),
        4,
        "rubric + one prominent value"
    );
    assert_eq!(
        (value.kind.as_str(), value.label.as_str()),
        (
            "Value",
            "How well does this plan satisfy \"Use unused space\"?"
        )
    );
    let ok = RateInput {
        proposal: plan.clone(),
        criterion: value.id.clone(),
        rating: Some(5),
    };
    let _: ActionHash = net.call(0, "governance", "rate", ok).await;
    let forged = RateInput {
        proposal: plan.clone(),
        criterion: format!("value:{}", ActionHashB64::from(plan.clone())),
        rating: Some(5),
    };
    let refused = net
        .try_call::<_, ActionHash>(0, "governance", "rate", forged)
        .await;
    assert!(refused.is_err(), "a plan is not a value");
    let board: PlanBoard = net.call(0, "governance", "list_plans", ()).await;
    let value = &board.plans[0].criteria[3];
    assert_eq!((value.count, value.mine), (1, Some(5)));
}

fn beds() -> Plan {
    Plan {
        kind: PlanKind::Production,
        title: "Beds".into(),
        description: "Six raised beds by the gate.".into(),
        demand_note: "Covers everyone who asked.".into(),
        subtype: Some("standard".into()),
        repository_url: None,
        location: Some("Allotment 4".into()),
        recipients: None,
        stages: vec![PlanStage {
            title: "Build".into(),
            details: "Cut and fill the beds.".into(),
            start_date: None,
            end_date: None,
        }],
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn values_are_rated_in_proposal_and_judge_the_plans() {
    // Two Full founders and one New member.
    let net = Net::new(3, 2, "values").await;
    net.demand_all().await;
    let space = rated_values(&net).await;
    criteria_on_a_plan(&net, &space).await;
}
