//! Help requests (ADR 0067): a commons space with mode `help-request` whose one activity the
//! asker writes. Helpers join and commit to one role each until a role is full; nobody else
//! adds activities, and the request takes no proposals and shows no phases.

use holochain::prelude::*;
use serde::{Deserialize, Serialize};

mod common;

use common::net::{charter, Charter, Net};

#[derive(Serialize, Debug)]
struct NewRole {
    label: String,
    required: u32,
    maximum: Option<u32>,
    description: String,
    suggest: Option<AgentPubKey>,
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
}

#[derive(Serialize, Debug)]
struct RoleInput {
    activity: ActionHash,
    role: u8,
}

#[derive(Deserialize, Debug)]
struct RoleView {
    description: String,
    maximum: Option<u32>,
    people: Vec<AgentPubKey>,
}

#[derive(Deserialize, Debug)]
struct ActivityView {
    roles: Vec<RoleView>,
}

#[derive(Deserialize, Debug)]
struct PhaseStep {}

#[derive(Deserialize, Debug)]
struct ProjectView {
    stage: String,
    phases: Vec<PhaseStep>,
    next_phase: Option<PhaseStep>,
}

fn help_charter() -> Charter {
    Charter {
        title: "Lift to the station".into(),
        description: "Need a lift on Friday morning.".into(),
        ..charter("help-request")
    }
}

fn request(roles: Vec<NewRole>) -> NewActivity {
    let at = |secs: i64| Timestamp::from_micros(Timestamp::now().as_micros() + secs * 1_000_000);
    NewActivity {
        title: "Lift to the station".into(),
        starts_at: at(3600),
        ends_at: at(7200),
        online: false,
        place: "Allotment gate".into(),
        meeting_url: None,
        note: "Need a lift on Friday morning.".into(),
        plan_stage: None,
        roles,
    }
}

fn role(label: &str, slots: Option<u32>) -> NewRole {
    NewRole {
        label: label.into(),
        required: slots.unwrap_or(1),
        maximum: slots,
        description: format!("{label} wanted"),
        suggest: None,
    }
}

async fn take(net: &Net, i: usize, activity: &ActionHash, role: u8) -> Result<ActionHash, String> {
    let input = RoleInput {
        activity: activity.clone(),
        role,
    };
    net.try_call(i, "activities", "take_role", input).await
}

#[tokio::test(flavor = "multi_thread")]
async fn helpers_commit_to_one_role_each() {
    // Agent 0 asks; agents 1 and 2 join (Net joins everyone but the founder).
    let net = Net::with_charter(3, 1, "help-requests", help_charter()).await;
    let view: ProjectView = net.call(0, "project", "get_project", ()).await;
    assert_eq!(view.stage, "Help request");
    assert!(view.phases.is_empty() && view.next_phase.is_none(), "no lifecycle");

    let roles = vec![role("Driver", Some(1)), role("Company", None)];
    let other: Result<ActionHash, String> = net
        .try_call(1, "activities", "create_activity", request(Vec::new()))
        .await;
    assert!(other.is_err(), "only the asker adds the activity");
    let lift: ActionHash = net
        .call(0, "activities", "create_activity", request(roles))
        .await;
    net.settle().await;

    take(&net, 1, &lift, 0).await.expect("driver");
    net.settle().await;
    assert!(take(&net, 2, &lift, 0).await.is_err(), "driver is full");
    assert!(take(&net, 1, &lift, 1).await.is_err(), "one role each");
    take(&net, 2, &lift, 1).await.expect("company");
    net.settle().await;
    let all: Vec<ActivityView> = net.call(0, "activities", "list_activities", ()).await;
    let seen = &all[0].roles;
    assert_eq!(seen[0].people.len() + seen[1].people.len(), 2);
    assert_eq!(seen[0].description, "Driver wanted");
    assert_eq!(seen[1].maximum, None, "any number");

    #[derive(Serialize, Debug)]
    enum Kind {
        Update { body: String },
    }
    let proposal: Result<ActionHash, String> = net
        .try_call(
            0,
            "governance",
            "propose",
            Kind::Update {
                body: "On my way".into(),
            },
        )
        .await;
    assert!(proposal.is_err(), "help requests take no proposals");
    net.shut_down().await;
}
