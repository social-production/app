//! M3b plans: plans are proposals members rate and approve; the leading approved plan gates
//! leaving a plan phase and chooses the subtype (ADR 0039).

use holochain::prelude::*;
use serde::{Deserialize, Serialize};
use sp_governance::plans::PlanKind;

mod common;

use common::net::{Charter, Net};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct PlanStage {
    title: String,
    details: String,
    start_date: Option<String>,
    end_date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Plan {
    kind: PlanKind,
    title: String,
    description: String,
    demand_note: String,
    subtype: Option<String>,
    repository_url: Option<String>,
    location: Option<String>,
    recipients: Option<String>,
    stages: Vec<PlanStage>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
enum ProposalKind {
    Phase {
        to: String,
        plan: Option<ActionHash>,
    },
    Plan(Plan),
}

#[derive(Serialize, Debug)]
struct VoteInput {
    proposal: ActionHash,
    choice: &'static str,
}

#[derive(Serialize, Debug)]
struct RateInput {
    proposal: ActionHash,
    criterion: String,
    rating: Option<u8>,
}

#[derive(Deserialize, Debug)]
struct Tally {
    yes: u32,
    status: String,
}

#[derive(Deserialize, Debug)]
struct Criterion {
    id: String,
    mine: Option<u8>,
    count: u32,
}

#[derive(Deserialize, Debug)]
struct PlanView {
    id: ActionHash,
    tally: Tally,
    approved: bool,
    leader: Option<String>,
    criteria: Vec<Criterion>,
}

#[derive(Deserialize, Debug)]
struct PlanBoard {
    plans: Vec<PlanView>,
    open_kind: Option<PlanKind>,
    gate: Option<PlanKind>,
    gate_leader: Option<ActionHash>,
}

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
}

#[derive(Deserialize, Debug)]
struct Listed {
    id: ActionHash,
}

fn plan(title: &str, subtype: &str) -> ProposalKind {
    ProposalKind::Plan(draft(PlanKind::Production, title, subtype))
}

fn draft(kind: PlanKind, title: &str, subtype: &str) -> Plan {
    Plan {
        kind,
        title: title.into(),
        description: "Six raised beds by the gate.".into(),
        demand_note: "Covers everyone who asked.".into(),
        subtype: Some(subtype.into()),
        repository_url: None,
        location: Some("Allotment 4".into()),
        recipients: None,
        stages: vec![PlanStage {
            title: "Build".into(),
            details: "Cut and fill the beds.".into(),
            start_date: Some("2026-11-01".into()),
            end_date: None,
        }],
    }
}

fn move_to(to: &str) -> ProposalKind {
    ProposalKind::Phase {
        to: to.into(),
        plan: None,
    }
}

async fn vote_yes(net: &Net, i: usize, proposal: &ActionHash) {
    let input = VoteInput {
        proposal: proposal.clone(),
        choice: "Yes",
    };
    let _: ActionHash = net.call(i, "governance", "vote", input).await;
}

async fn rate_all(net: &Net, i: usize, proposal: &ActionHash, rating: u8) {
    let board: PlanBoard = net.call(i, "governance", "list_plans", ()).await;
    let view = board
        .plans
        .iter()
        .find(|p| &p.id == proposal)
        .expect("plan");
    for c in &view.criteria {
        let input = RateInput {
            proposal: proposal.clone(),
            criterion: c.id.clone(),
            rating: Some(rating),
        };
        let _: ActionHash = net.call(i, "governance", "rate", input).await;
    }
}

async fn project(net: &Net, i: usize) -> ProjectView {
    let view: Option<ProjectView> = net.call(i, "project", "get_project", ()).await;
    view.expect("project view")
}

/// Two founders move the project into Production Plan.
async fn into_planning(seed: &str) -> Net {
    let net = Net::new(2, 2, seed).await;
    net.demand_all().await;
    let id: ActionHash = net
        .call(0, "governance", "propose", move_to("phase-2"))
        .await;
    net.settle().await;
    vote_yes(&net, 1, &id).await;
    net.settle().await;
    assert_eq!(project(&net, 0).await.phase, "phase-2");
    net
}

#[tokio::test(flavor = "multi_thread")]
async fn an_approved_plan_lets_the_project_leave_its_plan_phase() {
    let net = into_planning("plan-gate").await;
    let early = net
        .try_call::<_, ActionHash>(0, "governance", "propose", move_to("phase-3"))
        .await;
    assert!(early.is_err(), "no approved production plan yet");
    let wrong_phase = ProposalKind::Plan(draft(PlanKind::Distribution, "Hand out", "standard"));
    let refused = net
        .try_call::<_, ActionHash>(1, "governance", "propose", wrong_phase)
        .await;
    assert!(refused.is_err(), "distribution plans wait for phase-3");

    let id: ActionHash = net
        .call(1, "governance", "propose", plan("Beds", "standard"))
        .await;
    net.settle().await;
    let board: PlanBoard = net.call(0, "governance", "list_plans", ()).await;
    let view = &board.plans[0];
    assert_eq!(board.open_kind, Some(PlanKind::Production));
    assert_eq!(
        (view.tally.yes, view.criteria.len()),
        (0, 3),
        "no author vote"
    );
    let listed: Vec<Listed> = net.call(0, "governance", "list_proposals", ()).await;
    assert!(
        listed.iter().all(|p| p.id != id),
        "plans have their own board"
    );
    let bad = RateInput {
        proposal: id.clone(),
        criterion: "rubric:made-up".into(),
        rating: Some(3),
    };
    assert!(net
        .try_call::<_, ActionHash>(0, "governance", "rate", bad)
        .await
        .is_err());

    for i in 0..2 {
        rate_all(&net, i, &id, 4).await;
        vote_yes(&net, i, &id).await;
        net.settle().await;
    }
    let board: PlanBoard = net.call(0, "governance", "list_plans", ()).await;
    let view = &board.plans[0];
    assert_eq!(
        (
            view.approved,
            view.leader.as_deref(),
            view.tally.status.as_str()
        ),
        (true, Some("Leading"), "Passed")
    );
    assert_eq!(
        (view.criteria[0].count, view.criteria[0].mine),
        (2, Some(4))
    );
    assert_eq!(
        (board.gate, board.gate_leader.as_ref()),
        (Some(PlanKind::Production), Some(&id))
    );

    let next: ActionHash = net
        .call(0, "governance", "propose", move_to("phase-3"))
        .await;
    net.settle().await;
    vote_yes(&net, 1, &next).await;
    net.settle().await;
    let view = project(&net, 1).await;
    assert_eq!(
        (view.phase.as_str(), view.stage.as_str()),
        ("phase-3", "Distribution Plan")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn ratings_pick_the_leader_and_its_subtype_shapes_the_phases() {
    let net = into_planning("plan-leader").await;
    let plain: ActionHash = net
        .call(0, "governance", "propose", plan("Plain", "standard"))
        .await;
    let app: ActionHash = net
        .call(1, "governance", "propose", plan("App", "software"))
        .await;
    net.settle().await;
    for i in 0..2 {
        rate_all(&net, i, &plain, 3).await;
        rate_all(&net, i, &app, 5).await;
        vote_yes(&net, i, &plain).await;
        vote_yes(&net, i, &app).await;
        net.settle().await;
    }
    let board: PlanBoard = net.call(1, "governance", "list_plans", ()).await;
    let leader = board
        .plans
        .iter()
        .find(|p| p.leader.is_some())
        .expect("a leader");
    assert_eq!(leader.id, app, "same approval; the better rated plan leads");
    assert_eq!(
        leader.criteria.len(),
        4,
        "software plans add the repository question"
    );

    // The software plan skips distribution, as on web.
    let view = project(&net, 0).await;
    assert_eq!(view.charter.map(|c| c.subtype).as_deref(), Some("software"));
    assert_eq!(view.next_phase.map(|s| s.id).as_deref(), Some("phase-5"));
    let next: ActionHash = net
        .call(1, "governance", "propose", move_to("phase-5"))
        .await;
    net.settle().await;
    vote_yes(&net, 0, &next).await;
    net.settle().await;
    assert_eq!(project(&net, 0).await.stage, "Activity");
}
