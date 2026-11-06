//! A personal service's weekly hours and bookings (ADR 0077, web's
//! `projects_service_availability.py`): only its creator lists or removes hours; an accepted
//! timed request becomes a booking whose one Provider role the creator holds; once it has
//! ended the person who asked checks in and rates it like a participant.

use std::time::Duration;

use holochain::prelude::*;
use serde::{Deserialize, Serialize};

mod common;

use common::net::{charter, Net};

#[derive(Serialize, Debug, Clone)]
struct RuleInput {
    weekday: u8,
    start: u16,
    end: u16,
    zone: String,
    note: String,
}

#[derive(Deserialize, Debug)]
struct RuleView {
    weekday: u8,
    zone: String,
}

#[derive(Serialize, Debug, Clone)]
struct BookInput {
    request: ActionHash,
    requester: AgentPubKey,
    title: String,
    starts_at: Timestamp,
    ends_at: Timestamp,
}

#[derive(Deserialize, Debug)]
struct BookingView {
    requester: AgentPubKey,
}

#[derive(Deserialize, Debug)]
struct RoleView {
    label: String,
    people: Vec<AgentPubKey>,
}

#[derive(Deserialize, Debug)]
struct CompletionView {
    completed: u32,
    requester: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct RatingView {
    author: AgentPubKey,
    stars: u8,
}

#[derive(Deserialize, Debug)]
struct ActivityView {
    id: ActionHash,
    roles: Vec<RoleView>,
    booking: Option<BookingView>,
    completion: CompletionView,
    ratings: Vec<RatingView>,
}

#[derive(Serialize, Debug, Clone)]
struct CheckInInput {
    activity: ActionHash,
    done: Option<bool>,
}

#[derive(Serialize, Debug, Clone)]
struct RateInput {
    activity: ActionHash,
    stars: Option<u8>,
    comment: String,
}

fn weekly(weekday: u8) -> RuleInput {
    RuleInput {
        weekday,
        start: 9 * 60,
        end: 12 * 60,
        zone: "Australia/Melbourne".into(),
        note: "Mornings".into(),
    }
}

fn in_secs(secs: i64) -> Timestamp {
    Timestamp::from_micros(Timestamp::now().as_micros() + secs * 1_000_000)
}

fn agent(net: &Net, i: usize) -> AgentPubKey {
    net.cells[i].1.agent_pubkey().clone()
}

async fn rules(net: &Net, i: usize) -> Vec<RuleView> {
    net.call(i, "activities", "list_availability_rules", ())
        .await
}

#[tokio::test(flavor = "multi_thread")]
async fn only_the_creator_lists_weekly_hours() {
    let net = Net::with_charter(2, 1, "service-hours", charter("personal-service")).await;
    let other: Result<ActionHash, String> = net
        .try_call(1, "activities", "add_availability_rule", weekly(0))
        .await;
    assert!(
        other.is_err(),
        "only the service creator edits availability"
    );
    let bad: Result<ActionHash, String> = net
        .try_call(0, "activities", "add_availability_rule", weekly(7))
        .await;
    assert!(bad.is_err(), "weekday must be 0-6");
    let rule: ActionHash = net
        .call(0, "activities", "add_availability_rule", weekly(2))
        .await;
    net.settle().await;
    let seen = rules(&net, 1).await;
    assert_eq!(seen.len(), 1);
    assert_eq!(
        (seen[0].weekday, seen[0].zone.as_str()),
        (2, "Australia/Melbourne")
    );
    let theirs: Result<(), String> = net
        .try_call(1, "activities", "remove_availability_rule", rule.clone())
        .await;
    assert!(theirs.is_err(), "only the creator removes hours");
    let () = net
        .call(0, "activities", "remove_availability_rule", rule)
        .await;
    net.settle().await;
    assert!(
        rules(&net, 1).await.is_empty(),
        "removed hours stop showing"
    );
    net.shut_down().await;
}

async fn booking(net: &Net, i: usize) -> ActivityView {
    let all: Vec<ActivityView> = net.call(i, "activities", "list_activities", ()).await;
    all.into_iter()
        .find(|a| a.booking.is_some())
        .expect("the booking")
}

/// Agent 0 books agent 1's request for a few seconds from now.
async fn book(net: &Net) -> ActionHash {
    let input = BookInput {
        // Any hash stands in for the request in the direct chat.
        request: net.space.clone().expect("a commons space"),
        requester: agent(net, 1),
        title: "Fix my bike".into(),
        starts_at: in_secs(2),
        ends_at: in_secs(6),
    };
    let theirs: Result<ActionHash, String> = net
        .try_call(1, "activities", "book_request", input.clone())
        .await;
    assert!(theirs.is_err(), "only the creator books");
    let first: ActionHash = net
        .call(0, "activities", "book_request", input.clone())
        .await;
    let again: ActionHash = net.call(0, "activities", "book_request", input).await;
    assert_eq!(first, again, "a request is booked once");
    first
}

#[tokio::test(flavor = "multi_thread")]
async fn the_requester_checks_in_and_rates_a_booking() {
    let net = Net::with_charter(3, 1, "service-booking", charter("personal-service")).await;
    let id = book(&net).await;
    net.settle().await;
    let seen = booking(&net, 2).await;
    assert_eq!(seen.id, id);
    assert_eq!(
        seen.booking.as_ref().map(|b| &b.requester),
        Some(&agent(&net, 1))
    );
    assert_eq!(seen.roles[0].label, "Provider");
    assert_eq!(
        seen.roles[0].people,
        vec![agent(&net, 0)],
        "the creator provides"
    );
    tokio::time::sleep(Duration::from_secs(8)).await;
    let check = |done| CheckInInput {
        activity: id.clone(),
        done,
    };
    let rate = |stars| RateInput {
        activity: id.clone(),
        stars,
        comment: String::new(),
    };
    let outsider: Result<ActionHash, String> = net
        .try_call(2, "activities", "rate_activity", rate(Some(1)))
        .await;
    assert!(outsider.is_err(), "only participants or the requester rate");
    let _: ActionHash = net
        .call(1, "activities", "check_in", check(Some(true)))
        .await;
    let _: ActionHash = net
        .call(1, "activities", "rate_activity", rate(Some(5)))
        .await;
    let _: ActionHash = net
        .call(0, "activities", "check_in", check(Some(true)))
        .await;
    net.settle().await;
    let after = booking(&net, 2).await;
    assert_eq!(after.completion.requester, Some(true));
    assert_eq!(after.completion.completed, 1, "the provider checked in");
    let stars: Vec<(&AgentPubKey, u8)> =
        after.ratings.iter().map(|r| (&r.author, r.stars)).collect();
    assert_eq!(stars, vec![(&agent(&net, 1), 5)]);
    net.shut_down().await;
}
