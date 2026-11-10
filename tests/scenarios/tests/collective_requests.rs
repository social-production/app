//! Collective services' requests (ADR 0089): a request lives in a group chat between the
//! person asking and the service's members; any member decides it, the person asking can't,
//! and the first decision settles it. In the Activity phase a member plans a request into an
//! activity with the roles it needs (a booking naming the request); planning it again returns
//! that activity, and the person asking can't plan their own.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch};
use serde::{Deserialize, Serialize};
use sp_governance::plans::PlanKind;

mod common;

use common::chats::chat_network;
use common::net::{charter, Net};
use common::plans::{Plan, PlanBoard, PlanStage, ProposalKind, RateInput, VoteInput};

#[derive(Serialize, Debug)]
struct StartInput {
    members: Vec<AgentPubKey>,
    title: Option<String>,
}

#[derive(Serialize, Debug)]
struct RequestInput {
    project: String,
    service: String,
    title: String,
    body: String,
    starts_at: Option<Timestamp>,
    ends_at: Option<Timestamp>,
}

#[derive(Serialize, Debug)]
struct DecideInput {
    request: ActionHash,
    status: String,
    reason: String,
}

#[derive(Deserialize, Debug)]
struct RequestView {
    id: ActionHash,
    status: String,
    decided_by: Option<AgentPubKey>,
}

#[tokio::test(flavor = "multi_thread")]
async fn any_member_decides_a_request_in_the_group_chat() {
    let mut conductors = SweetConductorBatch::standard(3).await;
    // Agent 0 asks; agents 1 and 2 run the service together.
    let (cells, _) = chat_network(&mut conductors, "group").await;
    let start = StartInput {
        members: vec![
            cells[1].1.agent_pubkey().clone(),
            cells[2].1.agent_pubkey().clone(),
        ],
        title: Some("Request · Fix my bike".into()),
    };
    let () = conductors[0]
        .call(&cells[0].1.zome("messages"), "start_chat", start)
        .await;
    let ask = RequestInput {
        project: "uhCkkService".into(),
        service: "Bike co-op".into(),
        title: "Fix my bike".into(),
        body: "A flat tyre.".into(),
        starts_at: None,
        ends_at: None,
    };
    let fix: ActionHash = conductors[0]
        .call(&cells[0].1.zome("messages"), "send_service_request", ask)
        .await;
    let chats: Vec<_> = cells.iter().map(|(_, c)| c.clone()).collect();
    await_consistency(&chats).await.expect("consistency");

    let decide = |i: usize, status: &str| {
        let input = DecideInput {
            request: fix.clone(),
            status: status.into(),
            reason: String::new(),
        };
        let zome = cells[i].1.zome("messages");
        let conductor = &conductors[i];
        async move {
            conductor
                .call_fallible::<_, RequestView>(&zome, "decide_service_request", input)
                .await
                .map_err(|e| format!("{e:?}"))
        }
    };
    assert!(
        decide(0, "accepted").await.is_err(),
        "the person asking can't accept their own request"
    );
    let accepted = decide(2, "accepted").await.expect("any member accepts");
    assert_eq!(accepted.status, "accepted");
    assert_eq!(
        accepted.decided_by.as_ref(),
        Some(cells[2].1.agent_pubkey())
    );
    await_consistency(&chats).await.expect("consistency");
    assert!(
        decide(1, "declined").await.is_err(),
        "the first decision settles it"
    );
    let seen: Vec<RequestView> = conductors[1]
        .call(&cells[1].1.zome("messages"), "service_requests", ())
        .await;
    assert_eq!(seen.len(), 1);
    assert_eq!((&seen[0].id, seen[0].status.as_str()), (&fix, "accepted"));
    common::shut_down(conductors).await;
}

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
