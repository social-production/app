//! Collective services' requests (ADR 0089): a request lives in a group chat between the
//! person asking and the service's members; any member decides it, the person asking can't,
//! and the first decision settles it. Planning a request into an activity:
//! `collective_bookings.rs`.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch};
use serde::{Deserialize, Serialize};

mod common;

use common::chats::chat_network;

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

type Cells = Vec<(
    holochain::sweettest::SweetCell,
    holochain::sweettest::SweetCell,
)>;

/// Agent 0 starts a group chat with agents 1 and 2 (who run the service) and asks in it.
async fn ask_in_group(conductors: &SweetConductorBatch, cells: &Cells) -> ActionHash {
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
    let fix = conductors[0]
        .call(&cells[0].1.zome("messages"), "send_service_request", ask)
        .await;
    let chats: Vec<_> = cells.iter().map(|(_, c)| c.clone()).collect();
    await_consistency(&chats).await.expect("consistency");
    fix
}

#[tokio::test(flavor = "multi_thread")]
async fn any_member_decides_a_request_in_the_group_chat() {
    let mut conductors = SweetConductorBatch::standard(3).await;
    let (cells, _) = chat_network(&mut conductors, "group").await;
    let fix = ask_in_group(&conductors, &cells).await;
    let chats: Vec<_> = cells.iter().map(|(_, c)| c.clone()).collect();

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
