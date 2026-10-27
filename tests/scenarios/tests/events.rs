//! M5c events (ADR 0057): an event is a project clone with mode "event". It moves through
//! Proposal, Event Plan, Activity and Closed, and its plans carry a schedule and a place.

use holochain::prelude::*;
use holochain::sweettest::SweetConductorBatch;
use serde::{Deserialize, Serialize};
use sp_governance::plans::PlanKind;

mod common;

use common::net::{charter, Charter, Net};
use common::plans::{EventSchedule, Plan, PlanBoard, PlanStage, ProposalKind, VoteInput};

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

fn schedule() -> EventSchedule {
    EventSchedule {
        mode: "date".into(),
        start_date: Some("2026-11-14".into()),
        end_date: None,
        start_time: Some("09:00".into()),
        finish_time: Some("13:00".into()),
    }
}

fn event_plan(schedule: Option<EventSchedule>, location: Option<&str>) -> ProposalKind {
    ProposalKind::Plan(draft(schedule, location))
}

fn draft(schedule: Option<EventSchedule>, location: Option<&str>) -> Plan {
    Plan {
        kind: PlanKind::Event,
        title: "Saturday working bee".into(),
        description: "Weed, mulch and share lunch.".into(),
        demand_note: "Everyone who signalled can come.".into(),
        subtype: None,
        repository_url: None,
        location: location.map(Into::into),
        recipients: None,
        stages: vec![PlanStage {
            title: "Morning".into(),
            details: "Tools out at nine.".into(),
            start_date: None,
            end_date: None,
        }],
        schedule,
    }
}

fn move_to(to: &str) -> ProposalKind {
    ProposalKind::Phase {
        to: to.into(),
        plan: None,
        reason: "Ready for the next step.".into(),
    }
}

async fn vote_yes(net: &Net, i: usize, proposal: &ActionHash) {
    let input = VoteInput {
        proposal: proposal.clone(),
        choice: "Yes",
    };
    let _: ActionHash = net.call(i, "governance", "vote", input).await;
}

async fn event(net: &Net, i: usize) -> ProjectView {
    let view: Option<ProjectView> = net.call(i, "project", "get_project", ()).await;
    view.expect("event view")
}

/// Both founders vote a move through.
async fn pass(net: &Net, proposal: ProposalKind) {
    let id: ActionHash = net.call(0, "governance", "propose", proposal).await;
    net.settle().await;
    vote_yes(net, 1, &id).await;
    net.settle().await;
}

/// Event plans need a schedule and a place; project plans are refused in an event.
async fn plan_rules(net: &Net) {
    for (bad, why) in [
        (event_plan(None, Some("Allotment 4")), "no schedule"),
        (event_plan(Some(schedule()), None), "no place"),
    ] {
        let refused = net
            .try_call::<_, ActionHash>(0, "governance", "propose", bad)
            .await;
        assert!(refused.is_err(), "{why}");
    }
    let mut backwards = schedule();
    backwards.mode = "range".into();
    backwards.end_date = Some("2026-11-01".into());
    let refused = net
        .try_call::<_, ActionHash>(
            0,
            "governance",
            "propose",
            event_plan(Some(backwards), Some("Hall")),
        )
        .await;
    assert!(refused.is_err(), "a range ends after it starts");
    let mut production = draft(Some(schedule()), Some("Hall"));
    production.kind = PlanKind::Production;
    let refused = net
        .try_call::<_, ActionHash>(0, "governance", "propose", ProposalKind::Plan(production))
        .await;
    assert!(refused.is_err(), "events take event plans only");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_event_moves_through_webs_event_lifecycle() {
    let net = Net::with_charter(2, 2, "event-life", charter("event")).await;
    let view = event(&net, 1).await;
    assert_eq!(view.charter.map(|c| c.mode).as_deref(), Some("event"));
    assert_eq!(view.stage, "Proposal");

    net.demand_all().await;
    pass(&net, move_to("phase-2")).await;
    let view = event(&net, 0).await;
    assert_eq!(
        (view.phase.as_str(), view.stage.as_str()),
        ("phase-2", "Event Plan")
    );
    assert_eq!(view.next_phase.map(|s| s.id).as_deref(), Some("phase-5"));

    let early = net
        .try_call::<_, ActionHash>(0, "governance", "propose", move_to("phase-5"))
        .await;
    assert!(early.is_err(), "no approved event plan yet");
    plan_rules(&net).await;

    let id: ActionHash = net
        .call(
            1,
            "governance",
            "propose",
            event_plan(Some(schedule()), Some("Allotment 4")),
        )
        .await;
    net.settle().await;
    // Settle after each vote, so the last voter sees both and closes the proposal.
    for i in 0..2 {
        vote_yes(&net, i, &id).await;
        net.settle().await;
    }
    let board: PlanBoard = net.call(0, "governance", "list_plans", ()).await;
    let view = &board.plans[0];
    assert_eq!(board.open_kind, Some(PlanKind::Event));
    assert_eq!(view.criteria.len(), 4, "web's shared and event rubric");
    assert_eq!(
        (view.approved, view.leader.as_deref()),
        (true, Some("Leading"))
    );

    pass(&net, move_to("phase-5")).await;
    let view = event(&net, 1).await;
    assert_eq!(
        (view.phase.as_str(), view.stage.as_str()),
        ("phase-5", "Activity")
    );
    assert_eq!(view.next_phase.map(|s| s.id).as_deref(), Some("phase-7"));
    pass(&net, move_to("phase-7")).await;
    assert_eq!(event(&net, 0).await.stage, "Closed");
    net.shut_down().await;
}

/// Mirrors the identity zome's `ListingInput`.
#[derive(Serialize, Debug)]
struct ListingInput {
    network_seed: String,
    title: String,
    summary: String,
    mode: String,
    subtype: String,
    location_label: String,
    phase: String,
    tags: Vec<String>,
}

fn listing(net: &Net, mode: &str, nonce: &str) -> ListingInput {
    let me = holo_hash::AgentPubKeyB64::from(net.cells[0].0.agent_pubkey().clone());
    ListingInput {
        network_seed: format!("{me}.{nonce}"),
        title: "Working bee".into(),
        summary: "Weed and mulch.".into(),
        mode: mode.into(),
        subtype: "standard".into(),
        location_label: String::new(),
        phase: "phase-1".into(),
        tags: Vec::new(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn events_are_standard_and_listed_by_kind() {
    let mut conductors = SweetConductorBatch::standard(1).await;
    let cells = common::project_with_directory(&mut conductors, 1, "event-charter").await;
    let net = Net { conductors, cells };
    let mut odd = charter("event");
    odd.subtype = "software".into();
    let refused = net
        .try_call::<_, ActionHash>(0, "project", "create_charter", odd)
        .await;
    assert!(refused.is_err(), "events have no subtype");

    let directory = net.cells[0].0.zome("identity");
    let call = |input: ListingInput| {
        Box::pin(net.conductors[0].call_fallible::<_, ActionHash>(
            &directory,
            "create_listing",
            input,
        ))
    };
    assert!(
        call(listing(&net, "event", "a")).await.is_ok(),
        "events are listed"
    );
    assert!(
        call(listing(&net, "help-request", "b")).await.is_ok(),
        "later kinds need no directory change"
    );
    assert!(
        call(listing(&net, "Event!", "c")).await.is_err(),
        "kinds are slugs"
    );
    net.shut_down().await;
}
