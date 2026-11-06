//! Service requests (ADR 0076, web's personal-service requests): a request lives in the direct
//! chat between the person asking and the service's creator and posts web's lines there; the
//! creator accepts or declines, the person asking cancels, and the first decision settles it.
//! A personal service's creator saves its request settings at once; a collective service's
//! need a reason and a vote.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch};
use serde::{Deserialize, Serialize};

mod common;

use common::chats::chat_network;
use common::net::{charter, Net};

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
    decline_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct MessageView {
    body: String,
}

fn request(title: &str) -> RequestInput {
    RequestInput {
        project: "uhCkkService".into(),
        service: "Bike repairs".into(),
        title: title.into(),
        body: "My bike has a flat tyre.".into(),
        starts_at: None,
        ends_at: None,
    }
}

fn decide(request: &ActionHash, status: &str, reason: &str) -> DecideInput {
    DecideInput {
        request: request.clone(),
        status: status.into(),
        reason: reason.into(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_request_is_settled_once_in_the_direct_chat() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    // Agent 0 asks; agent 1 runs the service.
    let (cells, _) = chat_network(&mut conductors, "direct").await;
    let chats: Vec<_> = cells.iter().map(|(_, c)| c.clone()).collect();
    let chat = |i: usize| cells[i].1.zome("messages");
    let provider = cells[1].1.agent_pubkey().clone();
    let start = StartInput {
        members: vec![provider],
        title: None,
    };
    let () = conductors[0].call(&chat(0), "start_chat", start).await;

    let fix: ActionHash = conductors[0]
        .call(&chat(0), "send_service_request", request("Fix my bike"))
        .await;
    await_consistency(&chats).await.expect("consistency");
    let own: Result<RequestView, _> = conductors[0]
        .call_fallible(&chat(0), "decide_service_request", decide(&fix, "accepted", ""))
        .await;
    assert!(own.is_err(), "the person asking can't accept their own request");
    let declined: RequestView = conductors[1]
        .call(
            &chat(1),
            "decide_service_request",
            decide(&fix, "declined", "Away that week"),
        )
        .await;
    assert_eq!(declined.status, "declined");
    assert_eq!(declined.decline_reason.as_deref(), Some("Away that week"));
    let again: Result<RequestView, _> = conductors[1]
        .call_fallible(&chat(1), "decide_service_request", decide(&fix, "accepted", ""))
        .await;
    assert!(again.is_err(), "the first decision settles it");

    let tune: ActionHash = conductors[0]
        .call(&chat(0), "send_service_request", request("Tune my bike"))
        .await;
    await_consistency(&chats).await.expect("consistency");
    let theirs: Result<RequestView, _> = conductors[1]
        .call_fallible(&chat(1), "decide_service_request", decide(&tune, "cancelled", ""))
        .await;
    assert!(theirs.is_err(), "only the person asking cancels");
    let cancelled: RequestView = conductors[0]
        .call(&chat(0), "decide_service_request", decide(&tune, "cancelled", ""))
        .await;
    assert_eq!(cancelled.status, "cancelled");
    await_consistency(&chats).await.expect("consistency");

    let seen: Vec<RequestView> = conductors[1].call(&chat(1), "service_requests", ()).await;
    let states: Vec<(&ActionHash, &str)> = seen.iter().map(|r| (&r.id, r.status.as_str())).collect();
    assert_eq!(states, vec![(&tune, "cancelled"), (&fix, "declined")]);
    let lines: Vec<MessageView> = conductors[1].call(&chat(1), "list_messages", ()).await;
    let bodies: Vec<&str> = lines.iter().map(|m| m.body.as_str()).collect();
    assert!(bodies
        .iter()
        .any(|b| b.starts_with("Request \"Fix my bike\" · Pending\n\nMy bike has a flat tyre.")));
    assert!(bodies
        .iter()
        .any(|b| b.starts_with("Request \"Fix my bike\" · Declined\n\nReason: Away that week")));
    assert!(bodies
        .iter()
        .any(|b| b.starts_with("Request \"Tune my bike\" · Cancelled")));
    common::shut_down(conductors).await;
}

#[derive(Serialize, Debug)]
enum Kind {
    RequestSettings {
        enabled: bool,
        mode: String,
        off_schedule: bool,
        reason: String,
    },
}

#[derive(Deserialize, Debug, PartialEq)]
struct Requests {
    enabled: bool,
    mode: String,
    off_schedule: bool,
}

#[derive(Deserialize, Debug)]
struct ProjectView {
    requests: Requests,
}

fn settings(enabled: bool, mode: &str, reason: &str) -> Kind {
    Kind::RequestSettings {
        enabled,
        mode: mode.into(),
        off_schedule: false,
        reason: reason.into(),
    }
}

async fn requests_of(net: &Net, i: usize) -> Requests {
    let view: Option<ProjectView> = net.call(i, "project", "get_project", ()).await;
    view.expect("the project").requests
}

#[tokio::test(flavor = "multi_thread")]
async fn a_personal_service_creator_saves_request_settings_at_once() {
    let net = Net::with_charter(2, 1, "service-settings", charter("personal-service")).await;
    let defaults = Requests {
        enabled: true,
        mode: "both".into(),
        off_schedule: true,
    };
    assert_eq!(requests_of(&net, 1).await, defaults, "web's personal defaults");
    let other: Result<ActionHash, String> = net
        .try_call(1, "governance", "propose", settings(false, "direct", ""))
        .await;
    assert!(other.is_err(), "only the creator changes them");
    let _: ActionHash = net
        .call(0, "governance", "propose", settings(false, "direct", ""))
        .await;
    net.settle().await;
    let saved = Requests {
        enabled: false,
        mode: "direct".into(),
        off_schedule: false,
    };
    assert_eq!(requests_of(&net, 1).await, saved);
    net.shut_down().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_collective_service_votes_on_request_settings_with_a_reason() {
    let net = Net::with_charter(2, 1, "service-votes", charter("collective-service")).await;
    assert!(!requests_of(&net, 0).await.enabled, "off until a vote passes");
    let bare: Result<ActionHash, String> = net
        .try_call(0, "governance", "propose", settings(true, "both", ""))
        .await;
    assert!(bare.is_err(), "a governed change needs a reason");
    net.shut_down().await;
    let products = Net::new(1, 1, "service-product").await;
    let product: Result<ActionHash, String> = products
        .try_call(0, "governance", "propose", settings(true, "both", "Open up"))
        .await;
    assert!(product.is_err(), "only service projects take request settings");
    products.shut_down().await;
}
