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
use common::steps::step;

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

/// Agent `i` decides `request` in the chat; `Err` when the zome refuses.
async fn try_decide(
    conductors: &SweetConductorBatch,
    cells: &Cells,
    i: usize,
    request: &ActionHash,
    status: &str,
    reason: &str,
) -> Result<RequestView, String> {
    let zome = cells[i].1.zome("messages");
    let call = conductors[i].call_fallible(
        &zome,
        "decide_service_request",
        decide(request, status, reason),
    );
    step(&format!("agent {i} decides {status}"), call)
        .await
        .map_err(|e| format!("{e:?}"))
}

async fn send(conductors: &SweetConductorBatch, cells: &Cells, title: &str) -> ActionHash {
    let zome = cells[0].1.zome("messages");
    let call = conductors[0].call(&zome, "send_service_request", request(title));
    let hash = step(&format!("send \"{title}\""), call).await;
    let chats: Vec<_> = cells.iter().map(|(_, c)| c.clone()).collect();
    step("consistency after sending", await_consistency(&chats))
        .await
        .expect("consistency");
    hash
}

type Cells = Vec<(
    holochain::sweettest::SweetCell,
    holochain::sweettest::SweetCell,
)>;

#[tokio::test(flavor = "multi_thread")]
async fn a_request_is_settled_once_in_the_direct_chat() {
    let mut conductors = step("start two conductors", SweetConductorBatch::standard(2)).await;
    // Agent 0 asks; agent 1 runs the service.
    let (cells, _) = step("install the chat", chat_network(&mut conductors, "direct")).await;
    let start = StartInput {
        members: vec![cells[1].1.agent_pubkey().clone()],
        title: None,
    };
    let () = step(
        "start_chat",
        conductors[0].call(&cells[0].1.zome("messages"), "start_chat", start),
    )
    .await;

    let fix = send(&conductors, &cells, "Fix my bike").await;
    let own = try_decide(&conductors, &cells, 0, &fix, "accepted", "").await;
    assert!(
        own.is_err(),
        "the person asking can't accept their own request"
    );
    let declined = try_decide(&conductors, &cells, 1, &fix, "declined", "Away that week")
        .await
        .expect("the creator declines");
    assert_eq!(declined.status, "declined");
    assert_eq!(declined.decline_reason.as_deref(), Some("Away that week"));
    let again = try_decide(&conductors, &cells, 1, &fix, "accepted", "").await;
    assert!(again.is_err(), "the first decision settles it");

    let tune = send(&conductors, &cells, "Tune my bike").await;
    let theirs = try_decide(&conductors, &cells, 1, &tune, "cancelled", "").await;
    assert!(theirs.is_err(), "only the person asking cancels");
    let cancelled = try_decide(&conductors, &cells, 0, &tune, "cancelled", "")
        .await
        .expect("the person asking cancels");
    assert_eq!(cancelled.status, "cancelled");
    let chats: Vec<_> = cells.iter().map(|(_, c)| c.clone()).collect();
    step("consistency after deciding", await_consistency(&chats))
        .await
        .expect("consistency");

    let chat = cells[1].1.zome("messages");
    let seen: Vec<RequestView> = step(
        "service_requests",
        conductors[1].call(&chat, "service_requests", ()),
    )
    .await;
    let states: Vec<(&ActionHash, &str)> =
        seen.iter().map(|r| (&r.id, r.status.as_str())).collect();
    assert_eq!(states, vec![(&tune, "cancelled"), (&fix, "declined")]);
    let lines: Vec<MessageView> = step(
        "list_messages",
        conductors[1].call(&chat, "list_messages", ()),
    )
    .await;
    let has = |start: &str| lines.iter().any(|m| m.body.starts_with(start));
    assert!(has(
        "Request \"Fix my bike\" · Pending\n\nMy bike has a flat tyre."
    ));
    assert!(has(
        "Request \"Fix my bike\" · Declined\n\nReason: Away that week"
    ));
    assert!(has("Request \"Tune my bike\" · Cancelled"));
    step("shut down", common::shut_down(conductors)).await;
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
    let view: Option<ProjectView> =
        step("get_project", net.call(i, "project", "get_project", ())).await;
    view.expect("the project").requests
}

#[tokio::test(flavor = "multi_thread")]
async fn a_personal_service_creator_saves_request_settings_at_once() {
    let net = step(
        "start the personal service",
        Net::with_charter(2, 1, "service-settings", charter("personal-service")),
    )
    .await;
    let defaults = Requests {
        enabled: true,
        mode: "both".into(),
        off_schedule: true,
    };
    assert_eq!(
        requests_of(&net, 1).await,
        defaults,
        "web's personal defaults"
    );
    let other: Result<ActionHash, String> = step(
        "a member proposes",
        net.try_call(1, "governance", "propose", settings(false, "direct", "")),
    )
    .await;
    assert!(other.is_err(), "only the creator changes them");
    let _: ActionHash = step(
        "the creator saves",
        net.call(0, "governance", "propose", settings(false, "direct", "")),
    )
    .await;
    step("settle", net.settle()).await;
    let saved = Requests {
        enabled: false,
        mode: "direct".into(),
        off_schedule: false,
    };
    assert_eq!(requests_of(&net, 1).await, saved);
    step("shut down", net.shut_down()).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_collective_service_votes_on_request_settings_with_a_reason() {
    let net = step(
        "start the collective service",
        Net::with_charter(2, 1, "service-votes", charter("collective-service")),
    )
    .await;
    assert!(
        !requests_of(&net, 0).await.enabled,
        "off until a vote passes"
    );
    let bare: Result<ActionHash, String> = step(
        "propose without a reason",
        net.try_call(0, "governance", "propose", settings(true, "both", "")),
    )
    .await;
    assert!(bare.is_err(), "a governed change needs a reason");
    step("shut down the service", net.shut_down()).await;
    let products = step(
        "start a productive project",
        Net::new(1, 1, "service-product"),
    )
    .await;
    let product: Result<ActionHash, String> = step(
        "propose on a productive project",
        products.try_call(
            0,
            "governance",
            "propose",
            settings(true, "both", "Open up"),
        ),
    )
    .await;
    assert!(
        product.is_err(),
        "only service projects take request settings"
    );
    step("shut down the project", products.shut_down()).await;
}
