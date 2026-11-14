//! ADR 0059: a public project or event starts as a proposal with no type. In its Plan phase
//! it takes production, operations and event plans side by side; the leading approved plan
//! gives it its type when it leaves the phase, and a return to the Plan phase reopens it.

use holochain::prelude::*;
use serde::Deserialize;
use sp_governance::plans::PlanKind;

mod common;

use common::net::{charter, Charter, Net};
use common::plans::{
    EventSchedule, Plan, PlanBoard, PlanStage, ProposalKind, RateInput, VoteInput,
};

#[derive(Deserialize, Debug)]
struct Step {
    id: String,
}

#[derive(Deserialize, Debug)]
struct ProjectView {
    charter: Option<Charter>,
    phase: String,
    stage: String,
    next_phase: Option<Step>,
    requests: Requests,
}

#[derive(Deserialize, Debug)]
struct Requests {
    enabled: bool,
    mode: String,
    off_schedule: bool,
}

fn draft(kind: PlanKind, title: &str) -> ProposalKind {
    let event = kind == PlanKind::Event;
    ProposalKind::Plan(Plan {
        kind,
        title: title.into(),
        description: "What we will do, and how.".into(),
        demand_note: "Covers everyone who asked.".into(),
        subtype: (!event).then(|| "standard".into()),
        repository_url: None,
        location: Some("Allotment 4".into()),
        recipients: None,
        stages: vec![PlanStage {
            title: "Start".into(),
            details: "Get going.".into(),
            start_date: Some("2026-11-01".into()),
            end_date: None,
        }],
        schedule: event.then(|| EventSchedule {
            mode: "date".into(),
            start_date: Some("2026-11-14".into()),
            end_date: None,
            start_time: Some("09:00".into()),
            finish_time: Some("13:00".into()),
        }),
        // The operations plan switches requests on, by calendar and directly.
        requests: (kind == PlanKind::Organisation).then(|| "both".into()),
    })
}

fn move_to(to: &str) -> ProposalKind {
    ProposalKind::Phase {
        to: to.into(),
        plan: None,
        reason: "The plan is ready.".into(),
    }
}

async fn project(net: &Net, i: usize) -> ProjectView {
    let view: Option<ProjectView> = net.call(i, "project", "get_project", ()).await;
    view.expect("project view")
}

async fn mode(net: &Net, i: usize) -> String {
    project(net, i).await.charter.expect("charter").mode
}

/// Both founders pass `proposal`.
async fn pass(net: &Net, proposer: usize, kind: ProposalKind) -> ActionHash {
    let id: ActionHash = net.call(proposer, "governance", "propose", kind).await;
    net.settle().await;
    let input = VoteInput {
        proposal: id.clone(),
        choice: "Yes",
    };
    let _: ActionHash = net.call(1 - proposer, "governance", "vote", input).await;
    net.settle().await;
    id
}

/// Both founders rate every criterion of a plan and approve it.
async fn approve(net: &Net, plan: &ActionHash, rating: u8) {
    for i in 0..2 {
        let board: PlanBoard = net.call(i, "governance", "list_plans", ()).await;
        let view = board.plans.iter().find(|p| &p.id == plan).expect("plan");
        for c in &view.criteria {
            let input = RateInput {
                proposal: plan.clone(),
                criterion: c.id.clone(),
                rating: Some(rating),
            };
            let _: ActionHash = net.call(i, "governance", "rate", input).await;
        }
        let input = VoteInput {
            proposal: plan.clone(),
            choice: "Yes",
        };
        let _: ActionHash = net.call(i, "governance", "vote", input).await;
        net.settle().await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_proposal_takes_its_type_from_its_leading_plan() {
    let net = Net::with_charter(2, 2, "proposal-type", charter("proposal")).await;
    net.demand_all().await;
    assert_eq!(
        (mode(&net, 0).await.as_str(), project(&net, 0).await.stage),
        ("proposal", "Proposal".to_string())
    );
    pass(&net, 0, move_to("phase-2")).await;
    let view = project(&net, 1).await;
    assert_eq!(
        (view.phase.as_str(), view.stage.as_str()),
        ("phase-2", "Plan")
    );
    assert!(view.next_phase.is_none(), "no plan leads yet");
    let stuck = net
        .try_call::<_, ActionHash>(0, "governance", "propose", move_to("phase-5"))
        .await;
    assert!(
        stuck.is_err(),
        "leaving the Plan phase needs a leading plan"
    );
    let late = net
        .try_call::<_, ActionHash>(
            0,
            "governance",
            "propose",
            draft(PlanKind::Distribution, "Hand out"),
        )
        .await;
    assert!(late.is_err(), "second-phase plans give no type");

    the_better_plan_gives_the_type(&net).await;
    a_return_reopens_the_type(&net).await;
    net.shut_down().await;
}

async fn the_better_plan_gives_the_type(net: &Net) {
    // An event plan and an operations plan compete; the better-rated operations plan leads.
    let day: ActionHash = net
        .call(
            1,
            "governance",
            "propose",
            draft(PlanKind::Event, "Swap day"),
        )
        .await;
    let service: ActionHash = net
        .call(
            0,
            "governance",
            "propose",
            draft(PlanKind::Organisation, "Run the seed library"),
        )
        .await;
    net.settle().await;
    approve(net, &day, 2).await;
    approve(net, &service, 5).await;
    let board: PlanBoard = net.call(1, "governance", "list_plans", ()).await;
    assert_eq!(board.gate_leader.as_ref(), Some(&service));
    let leading = |id: &ActionHash| {
        board
            .plans
            .iter()
            .find(|p| &p.id == id)
            .and_then(|p| p.leader.clone())
    };
    assert_eq!(
        (leading(&service).as_deref(), leading(&day)),
        (Some("Leading"), None),
        "one leader across the kinds"
    );
    let view = project(net, 1).await;
    assert_eq!(mode(net, 1).await, "proposal", "still a proposal");
    assert_eq!(
        view.next_phase.map(|s| s.id).as_deref(),
        Some("phase-5"),
        "a collective service's next phase"
    );

    pass(net, 0, move_to("phase-5")).await;
    let view = project(net, 1).await;
    assert_eq!(
        (view.phase.as_str(), view.stage.as_str()),
        ("phase-5", "Activity")
    );
    assert_eq!(mode(net, 1).await, "collective-service");
    assert!(
        view.requests.enabled && view.requests.mode == "both" && view.requests.off_schedule,
        "the cited operations plan switched requests on: {:?}",
        view.requests
    );
}

async fn a_return_reopens_the_type(net: &Net) {
    // Returning to the Plan phase reopens the type.
    let back = ProposalKind::Return {
        to: "phase-2".into(),
        reason: "The swap day fits better.".into(),
    };
    // A return comes without its author's vote (web's revert), so both founders vote.
    let back = pass(net, 1, back).await;
    let own = VoteInput {
        proposal: back,
        choice: "Yes",
    };
    let _: ActionHash = net.call(1, "governance", "vote", own).await;
    net.settle().await;
    let view = project(net, 0).await;
    assert_eq!(
        (view.phase.as_str(), view.stage.as_str()),
        ("phase-2", "Plan")
    );
    assert_eq!(mode(net, 0).await, "proposal");
    assert!(!view.requests.enabled, "a proposal takes no requests");
}
