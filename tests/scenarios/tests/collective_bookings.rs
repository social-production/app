//! Collective services' bookings (ADR 0089): in the Activity phase any member plans a request
//! into an activity with the roles it needs (a booking naming the request); planning it again
//! returns that activity, and the person asking can't plan their own. The request itself:
//! `collective_requests.rs`.

use holochain::prelude::*;
use serde::{Deserialize, Serialize};
use sp_governance::plans::PlanKind;

mod common;

use common::net::{charter, Net};
use common::plans::{Plan, PlanBoard, PlanStage, ProposalKind, RateInput, VoteInput};

#[derive(Serialize, Debug)]
struct NewRole {
    label: String,
    required: u32,
    maximum: Option<u32>,
    suggest: Option<AgentPubKey>,
}

#[derive(Serialize, Debug, Clone)]
struct Booking {
    request: ActionHash,
    requester: AgentPubKey,
}

#[derive(Serialize, Debug)]
struct NewActivity {
    title: String,
    starts_at: Timestamp,
    ends_at: Timestamp,
    online: bool,
    place: String,
    meeting_url: Option<String>,
    note: String,
    plan_stage: Option<String>,
    roles: Vec<NewRole>,
    booking: Option<Booking>,
}

#[derive(Deserialize, Debug)]
struct BookingView {
    requester: AgentPubKey,
}

#[derive(Deserialize, Debug)]
struct RoleView {
    label: String,
}

#[derive(Deserialize, Debug)]
struct ActivityView {
    id: ActionHash,
    roles: Vec<RoleView>,
    booking: Option<BookingView>,
}

#[derive(Deserialize, Debug)]
struct ProjectView {
    stage: String,
}

fn agent(net: &Net, i: usize) -> AgentPubKey {
    net.cells[i].1.agent_pubkey().clone()
}

async fn propose(net: &Net, kind: ProposalKind) -> ActionHash {
    let id = net.call(0, "governance", "propose", kind).await;
    net.settle().await;
    id
}

/// Everyone votes yes (the proposer's yes comes with the proposal).
async fn all_vote_yes(net: &Net, proposal: &ActionHash) {
    for i in 1..net.cells.len() {
        let input = VoteInput {
            proposal: proposal.clone(),
            choice: "Yes",
        };
        let _: ActionHash = net.call(i, "governance", "vote", input).await;
    }
    net.settle().await;
}

fn move_to(to: &str) -> ProposalKind {
    ProposalKind::Phase {
        to: to.into(),
        plan: None,
        reason: "We are ready.".into(),
    }
}

/// Three founders take a collective service through its operations plan to Activity.
async fn collective_in_activity() -> Net {
    let net = Net::with_charter(3, 3, "collective-requests", charter("collective-service")).await;
    net.demand_all().await;
    let to_plans = propose(&net, move_to("phase-2")).await;
    all_vote_yes(&net, &to_plans).await;
    let plan = propose(
        &net,
        ProposalKind::Plan(Plan {
            kind: PlanKind::Organisation,
            title: "Repair evenings".into(),
            description: "Members fix bikes on Thursday evenings.".into(),
            demand_note: "Everyone who asked.".into(),
            subtype: Some("standard".into()),
            repository_url: None,
            location: Some("The shed".into()),
            recipients: None,
            stages: vec![PlanStage {
                title: "Open".into(),
                details: "Open the shed.".into(),
                start_date: Some("2026-11-10".into()),
                end_date: None,
            }],
            schedule: None,
        }),
    )
    .await;
    let board: PlanBoard = net.call(0, "governance", "list_plans", ()).await;
    for i in 0..net.cells.len() {
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
    let to_activity = propose(&net, move_to("phase-5")).await;
    all_vote_yes(&net, &to_activity).await;
    let view: Option<ProjectView> = net.call(1, "project", "get_project", ()).await;
    assert_eq!(view.expect("project").stage, "Activity");
    net
}

fn planned(request: &ActionHash, requester: AgentPubKey) -> NewActivity {
    let now = Timestamp::now().as_micros();
    let role = |label: &str| NewRole {
        label: label.into(),
        required: 1,
        maximum: Some(1),
        suggest: None,
    };
    NewActivity {
        title: "Fix a flat tyre".into(),
        starts_at: Timestamp::from_micros(now + 3_600_000_000),
        ends_at: Timestamp::from_micros(now + 7_200_000_000),
        online: false,
        place: "The shed".into(),
        meeting_url: None,
        note: "A flat tyre.".into(),
        plan_stage: None,
        roles: vec![role("Service lead"), role("Helper")],
        booking: Some(Booking {
            request: request.clone(),
            requester,
        }),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_member_plans_a_request_into_an_activity_with_roles() {
    let net = collective_in_activity().await;
    // Any hash stands in for the request in the group chat.
    let request = net.space.clone().expect("a commons space");
    let own: Result<ActionHash, String> = net
        .try_call(
            2,
            "activities",
            "create_activity",
            planned(&request, agent(&net, 2)),
        )
        .await;
    assert!(
        own.is_err(),
        "the person asking can't plan their own request"
    );
    let first: ActionHash = net
        .call(
            1,
            "activities",
            "create_activity",
            planned(&request, agent(&net, 2)),
        )
        .await;
    net.settle().await;
    let again: ActionHash = net
        .call(
            0,
            "activities",
            "create_activity",
            planned(&request, agent(&net, 2)),
        )
        .await;
    assert_eq!(first, again, "a request is planned once");
    let all: Vec<ActivityView> = net.call(2, "activities", "list_activities", ()).await;
    let booked = all
        .iter()
        .find(|a| a.id == first)
        .expect("the planned activity");
    assert_eq!(
        booked.booking.as_ref().map(|b| &b.requester),
        Some(&agent(&net, 2))
    );
    let labels: Vec<&str> = booked.roles.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(labels, vec!["Service lead", "Helper"], "the roles it needs");
    net.shut_down().await;
}
