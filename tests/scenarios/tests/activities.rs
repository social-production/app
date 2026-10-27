//! Activities and roles (ADR 0056): members schedule activities in the Activity phase, take
//! one role each until a role is full, leave it, suggest people, and rate an activity they
//! took part in once it has ended. A personal service's Activity phase is its first phase,
//! so these run there; a productive project refuses activities while it is in Proposal.

use std::time::Duration;

use holochain::prelude::*;
use serde::{Deserialize, Serialize};

mod common;

use common::net::{charter, Net};

#[derive(Serialize, Debug)]
struct NewRole {
    label: String,
    required: u32,
    maximum: Option<u32>,
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

#[derive(Serialize, Debug)]
struct RateInput {
    activity: ActionHash,
    stars: Option<u8>,
    comment: String,
}

#[derive(Deserialize, Debug)]
struct SuggestionView {
    id: ActionHash,
    person: AgentPubKey,
    by: AgentPubKey,
}

#[derive(Deserialize, Debug)]
struct RoleView {
    people: Vec<AgentPubKey>,
    suggestion: Option<SuggestionView>,
    declined: Vec<SuggestionView>,
}

#[derive(Deserialize, Debug)]
struct RatingView {
    stars: u8,
    comment: String,
}

#[derive(Deserialize, Debug)]
struct CompletionView {
    eligible: u32,
    completed: u32,
}

#[derive(Deserialize, Debug)]
struct ActivityView {
    id: ActionHash,
    roles: Vec<RoleView>,
    my_role: Option<u8>,
    ratings: Vec<RatingView>,
    completion: CompletionView,
}

#[derive(Serialize, Debug)]
struct CheckInInput {
    activity: ActionHash,
    done: Option<bool>,
}

fn in_secs(secs: i64) -> Timestamp {
    Timestamp::from_micros(Timestamp::now().as_micros() + secs * 1_000_000)
}

fn role(label: &str, maximum: Option<u32>) -> NewRole {
    NewRole {
        label: label.into(),
        required: 1,
        maximum,
        suggest: None,
    }
}

fn activity(title: &str, starts: i64, ends: i64, roles: Vec<NewRole>) -> NewActivity {
    NewActivity {
        title: title.into(),
        starts_at: in_secs(starts),
        ends_at: in_secs(ends),
        online: false,
        place: "Shed".into(),
        meeting_url: None,
        note: "Bring tools.".into(),
        plan_stage: None,
        roles,
    }
}

async fn create(net: &Net, i: usize, a: NewActivity) -> Result<ActionHash, String> {
    net.try_call(i, "activities", "create_activity", a).await
}

async fn take(net: &Net, i: usize, activity: &ActionHash, role: u8) -> Result<ActionHash, String> {
    let input = RoleInput {
        activity: activity.clone(),
        role,
    };
    net.try_call(i, "activities", "take_role", input).await
}

async fn view(net: &Net, i: usize, id: &ActionHash) -> ActivityView {
    let all: Vec<ActivityView> = net.call(i, "activities", "list_activities", ()).await;
    all.into_iter().find(|a| a.id == *id).expect("the activity")
}

fn agent(net: &Net, i: usize) -> AgentPubKey {
    net.cells[i].1.agent_pubkey().clone()
}

/// Only the person offering a service schedules; roles fill up; one role each; leaving.
async fn roles_fill_and_free(net: &Net) -> ActionHash {
    let roles = vec![role("Mechanic", Some(1)), role("Helper", None)];
    assert!(
        create(net, 1, activity("Repairs", 60, 3600, Vec::new()))
            .await
            .is_err(),
        "not the author"
    );
    assert!(
        create(net, 0, activity("Past", -60, 60, Vec::new()))
            .await
            .is_err(),
        "starts in the past"
    );
    let repairs = create(net, 0, activity("Repairs", 60, 3600, roles))
        .await
        .expect("created");
    net.settle().await;

    take(net, 1, &repairs, 0).await.expect("took mechanic");
    net.settle().await;
    assert!(take(net, 2, &repairs, 0).await.is_err(), "mechanic is full");
    assert!(
        take(net, 1, &repairs, 1).await.is_err(),
        "one role per activity"
    );
    take(net, 2, &repairs, 1).await.expect("took helper");
    net.settle().await;
    let seen = view(net, 0, &repairs).await;
    assert_eq!(seen.roles[0].people, vec![agent(net, 1)]);
    assert_eq!(seen.roles[1].people, vec![agent(net, 2)]);
    assert_eq!(view(net, 1, &repairs).await.my_role, Some(0));

    let () = net
        .call(1, "activities", "leave_role", repairs.clone())
        .await;
    net.settle().await;
    assert!(
        view(net, 0, &repairs).await.roles[0].people.is_empty(),
        "left"
    );
    take(net, 2, &repairs, 0)
        .await
        .expect_err("still has the helper role");
    repairs
}

/// The latest suggestion per role stands until its person declines it.
async fn suggestions(net: &Net, repairs: &ActionHash) {
    #[derive(Serialize, Debug)]
    struct SuggestInput {
        activity: ActionHash,
        role: u8,
        person: AgentPubKey,
    }
    let to = |person| SuggestInput {
        activity: repairs.clone(),
        role: 0,
        person,
    };
    let mine: Result<ActionHash, String> = net
        .try_call(1, "activities", "suggest_role", to(agent(net, 1)))
        .await;
    assert!(mine.is_err(), "suggest someone else");
    let _: ActionHash = net
        .call(0, "activities", "suggest_role", to(agent(net, 1)))
        .await;
    net.settle().await;
    let s = view(net, 1, repairs).await.roles[0]
        .suggestion
        .take()
        .expect("suggested");
    assert_eq!(s.person, agent(net, 1));
    let by_other: Result<ActionHash, String> = net
        .try_call(2, "activities", "decline_suggestion", s.id.clone())
        .await;
    assert!(by_other.is_err(), "only the suggested person declines");
    let _: ActionHash = net.call(1, "activities", "decline_suggestion", s.id).await;
    net.settle().await;
    let role = &view(net, 0, repairs).await.roles[0];
    assert!(role.suggestion.is_none(), "declined");
    assert_eq!(role.declined.len(), 1);
    assert_eq!(role.declined[0].person, agent(net, 1));
    assert_eq!(role.declined[0].by, agent(net, 0));
}

/// Participants rate once it has ended; roles lock then.
async fn ratings(net: &Net) {
    let short = create(
        net,
        0,
        activity("Quick fix", 2, 20, vec![role("Helper", None)]),
    )
    .await
    .expect("created");
    net.settle().await;
    take(net, 1, &short, 0).await.expect("took");
    let rate = |stars| RateInput {
        activity: short.clone(),
        stars,
        comment: "Good day".into(),
    };
    let early: Result<ActionHash, String> = net
        .try_call(1, "activities", "rate_activity", rate(Some(5)))
        .await;
    assert!(early.is_err(), "not ended yet");
    tokio::time::sleep(Duration::from_secs(22)).await;
    assert!(
        take(net, 2, &short, 0).await.is_err(),
        "roles lock at the end"
    );
    let outsider: Result<ActionHash, String> = net
        .try_call(2, "activities", "rate_activity", rate(Some(4)))
        .await;
    assert!(outsider.is_err(), "only participants rate");
    let _: ActionHash = net
        .call(1, "activities", "rate_activity", rate(Some(5)))
        .await;
    let _: ActionHash = net
        .call(1, "activities", "rate_activity", rate(Some(4)))
        .await;
    net.settle().await;
    let check_in = |done| CheckInInput {
        activity: short.clone(),
        done,
    };
    let not_assigned: Result<ActionHash, String> = net
        .try_call(2, "activities", "check_in", check_in(Some(true)))
        .await;
    assert!(not_assigned.is_err(), "only participants check in");
    let _: ActionHash = net
        .call(1, "activities", "check_in", check_in(Some(true)))
        .await;
    net.settle().await;
    let seen = view(net, 0, &short).await;
    assert_eq!(
        (seen.completion.eligible, seen.completion.completed),
        (1, 1)
    );
    assert_eq!(seen.ratings.len(), 1, "the latest counts");
    assert_eq!(
        (seen.ratings[0].stars, seen.ratings[0].comment.as_str()),
        (4, "Good day")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn members_schedule_activities_take_roles_and_rate_them() {
    let net = Net::with_charter(3, 1, "activities", charter("personal-service")).await;
    let repairs = roles_fill_and_free(&net).await;
    suggestions(&net, &repairs).await;
    ratings(&net).await;
    net.shut_down().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_productive_project_waits_for_its_activity_phase() {
    let net = Net::new(2, 1, "activities-phase").await;
    let early = create(&net, 0, activity("Too soon", 60, 3600, Vec::new())).await;
    assert!(early.is_err(), "Proposal phase");
    net.shut_down().await;
}
