//! M5c private events (ADR 0061): the seed says invite-only (`.private.`) or organizer-run
//! (`.organized.`). Organizers are the creator plus `Organizer` grants; in an organizer-run
//! event only they propose and add activities, their proposals pass at once, and signals and
//! values are refused.

use holochain::prelude::*;
use serde::{Deserialize, Serialize};
use sp_governance::plans::PlanKind;

mod common;

use common::net::{charter, Net};
use common::plans::{EventSchedule, Plan, PlanBoard, PlanStage, ProposalKind};

#[derive(Deserialize, Debug)]
struct ProjectView {
    phase: String,
    audience: String,
    governance: String,
    organizers: Vec<AgentPubKey>,
    am_organizer: bool,
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
    roles: Vec<()>,
}

fn activity() -> NewActivity {
    let at = |hours: i64| Timestamp::from_micros(Timestamp::now().as_micros() + hours * 3_600_000_000);
    NewActivity {
        title: "Set up the hall".into(),
        starts_at: at(24),
        ends_at: at(26),
        online: false,
        place: "Hall".into(),
        meeting_url: None,
        note: "Bring a table.".into(),
        plan_stage: None,
        roles: Vec::new(),
    }
}

fn event_plan() -> ProposalKind {
    ProposalKind::Plan(Plan {
        kind: PlanKind::Event,
        title: "Birthday lunch".into(),
        description: "Lunch, then cake.".into(),
        demand_note: "Set by the organizers; this event takes no support signals.".into(),
        subtype: None,
        repository_url: None,
        location: Some("Hall".into()),
        recipients: None,
        stages: vec![PlanStage {
            title: "Lunch".into(),
            details: "Everyone brings a plate.".into(),
            start_date: None,
            end_date: None,
        }],
        schedule: Some(EventSchedule {
            mode: "date".into(),
            start_date: Some("2026-12-05".into()),
            end_date: None,
            start_time: Some("12:00".into()),
            finish_time: Some("15:00".into()),
        }),
    })
}

fn move_to(to: &str) -> ProposalKind {
    ProposalKind::Phase {
        to: to.into(),
        plan: None,
        reason: "The organizers are ready.".into(),
    }
}

async fn view(net: &Net, i: usize) -> ProjectView {
    let v: Option<ProjectView> = net.call(i, "project", "get_project", ()).await;
    v.expect("event view")
}

async fn propose(net: &Net, i: usize, kind: ProposalKind) -> Result<ActionHash, String> {
    net.try_call(i, "governance", "propose", kind).await
}

/// Agent 0 creates; agent 1 becomes a co-organizer; agent 2 only takes part.
#[tokio::test(flavor = "multi_thread")]
async fn organizers_run_an_organizer_controlled_event() {
    let net = Net::with_charter(3, 2, "organized.lunch", charter("event")).await;
    let v = view(&net, 2).await;
    assert_eq!(
        (v.audience.as_str(), v.governance.as_str()),
        ("invite_only", "organizer_controlled")
    );
    assert!(view(&net, 0).await.am_organizer && !v.am_organizer);

    let signal = net
        .try_call::<_, ()>(2, "project", "set_signal", Some("Demand"))
        .await;
    assert!(signal.is_err(), "no signals here");
    let value = net
        .try_call::<_, ActionHash>(2, "governance", "add_value", "Fun".to_string())
        .await;
    assert!(value.is_err(), "no values here");
    assert!(propose(&net, 2, move_to("phase-2")).await.is_err(), "members don't propose");

    let agent1 = net.cells[1].1.agent_pubkey().clone();
    let _: ActionHash = net.call(0, "project", "add_organizer", agent1).await;
    let refused = net
        .try_call::<_, ActionHash>(2, "project", "add_organizer", net.cells[2].1.agent_pubkey().clone())
        .await;
    assert!(refused.is_err(), "only organizers add organizers");
    net.settle().await;
    let v = view(&net, 1).await;
    assert!(v.am_organizer && v.organizers.len() == 2);

    // A co-organizer's move passes at once, without votes or signals.
    propose(&net, 1, move_to("phase-2")).await.expect("co-organizer moves");
    net.settle().await;
    assert_eq!(view(&net, 2).await.phase, "phase-2");

    propose(&net, 0, event_plan()).await.expect("organizer plan");
    net.settle().await;
    let board: PlanBoard = net.call(2, "governance", "list_plans", ()).await;
    assert!(board.plans[0].approved, "the organizer's plan is approved");
    assert_eq!(board.plans[0].leader.as_deref(), Some("Leading"));
    propose(&net, 0, move_to("phase-5")).await.expect("into Activity");
    net.settle().await;
    assert_eq!(view(&net, 2).await.phase, "phase-5");

    let back = ProposalKind::Return {
        to: "phase-1".into(),
        reason: "Start over.".into(),
    };
    assert!(propose(&net, 0, back).await.is_err(), "no way back to Proposal");
    let member = net
        .try_call::<_, ActionHash>(2, "activities", "create_activity", activity())
        .await;
    assert!(member.is_err(), "only organizers add activities");
    let organizer = net
        .try_call::<_, ActionHash>(1, "activities", "create_activity", activity())
        .await;
    assert!(organizer.is_ok(), "{organizer:?}");
    net.shut_down().await;
}

/// A collaborative private event takes organizers but still votes; a public one has none.
#[tokio::test(flavor = "multi_thread")]
async fn private_collaborative_events_still_vote() {
    let net = Net::with_charter(2, 2, "private.picnic", charter("event")).await;
    let v = view(&net, 1).await;
    assert_eq!(
        (v.audience.as_str(), v.governance.as_str()),
        ("invite_only", "collaborative")
    );
    let agent1 = net.cells[1].1.agent_pubkey().clone();
    let _: ActionHash = net.call(0, "project", "add_organizer", agent1).await;
    net.demand_all().await;
    propose(&net, 0, move_to("phase-2")).await.expect("proposed");
    net.settle().await;
    assert_eq!(view(&net, 1).await.phase, "phase-1", "waits for the second vote");
    net.shut_down().await;

    let public = Net::with_charter(2, 2, "fair", charter("event")).await;
    let agent1 = public.cells[1].1.agent_pubkey().clone();
    let refused = public
        .try_call::<_, ActionHash>(0, "project", "add_organizer", agent1)
        .await;
    assert!(refused.is_err(), "public events have no co-organizers");
    public.shut_down().await;
}
