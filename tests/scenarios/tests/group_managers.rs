//! Co-managers and a group photo (ADR 0122): any manager makes another member a manager,
//! changes the photo and runs the group; others can't; a manager who leaves hands over only
//! when they were the last, and the last can't step down.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetCell, SweetConductorBatch};
use serde::{Deserialize, Serialize};

mod common;

use common::chats::chat_network;

#[derive(Serialize, Debug)]
struct StartInput {
    members: Vec<AgentPubKey>,
    title: Option<String>,
}

#[derive(Serialize, Debug)]
struct ManagerInput {
    agent: AgentPubKey,
    on: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Bytes(#[serde(with = "serde_bytes")] Vec<u8>);

#[derive(Deserialize, Debug)]
struct ChatSummary {
    managers: Vec<AgentPubKey>,
    members: Vec<AgentPubKey>,
    title: Option<String>,
    picture: Option<Bytes>,
}

type Cells = Vec<(SweetCell, SweetCell)>;

async fn settle(cells: &Cells) {
    let chats: Vec<SweetCell> = cells.iter().map(|(_, c)| c.clone()).collect();
    await_consistency(&chats).await.expect("chat consistency");
}

#[tokio::test(flavor = "multi_thread")]
async fn co_managers_run_the_group_and_set_its_photo() {
    let mut conductors = SweetConductorBatch::standard(4).await;
    let (cells, _) = chat_network(&mut conductors, "group").await;
    let agent = |i: usize| cells[i].1.agent_pubkey().clone();
    let chat = |i: usize| cells[i].1.zome("messages");
    let grant = |i: usize, on: bool| ManagerInput {
        agent: agent(i),
        on,
    };
    let start = StartInput {
        members: vec![agent(1), agent(2)],
        title: Some("Crew".into()),
    };
    let () = conductors[0].call(&chat(0), "start_chat", start).await;
    settle(&cells).await;

    let cats: Result<(), _> = conductors[2]
        .call_fallible(&chat(2), "set_manager", grant(2, true))
        .await;
    assert!(cats.is_err(), "nobody makes themselves a manager");
    let () = conductors[0]
        .call(&chat(0), "set_manager", grant(1, true))
        .await;
    settle(&cells).await;
    let () = conductors[1].call(&chat(1), "add_member", agent(3)).await;
    let photo = Bytes(vec![0xff, 0xd8, 0xff, 0xe0, 1, 2, 3]);
    let () = conductors[1]
        .call(&chat(1), "set_picture", Bytes(photo.0.clone()))
        .await;
    let cats: Result<(), _> = conductors[2]
        .call_fallible(&chat(2), "set_picture", Bytes(vec![1]))
        .await;
    assert!(cats.is_err(), "only a manager sets the photo");
    settle(&cells).await;
    let cat: ChatSummary = conductors[2].call(&chat(2), "summary", ()).await;
    assert_eq!(cat.managers, vec![agent(0), agent(1)]);
    assert_eq!(cat.members, vec![agent(0), agent(1), agent(2), agent(3)]);
    assert_eq!(cat.picture, Some(photo));

    handing_on(&conductors, &cells).await;
    common::shut_down(conductors).await;
}

/// Ann leaves while Bob still manages; Bob can't step down alone, makes Cat a manager, then
/// steps down.
async fn handing_on(conductors: &SweetConductorBatch, cells: &Cells) {
    let agent = |i: usize| cells[i].1.agent_pubkey().clone();
    let chat = |i: usize| cells[i].1.zome("messages");
    let grant = |i: usize, on: bool| ManagerInput {
        agent: agent(i),
        on,
    };
    // Ann leaves: Bob still manages, so nobody is handed the group.
    let () = conductors[0].call(&chat(0), "leave", ()).await;
    settle(cells).await;
    let bob: ChatSummary = conductors[1].call(&chat(1), "summary", ()).await;
    assert_eq!(bob.managers, vec![agent(1)]);
    let steps_down: Result<(), _> = conductors[1]
        .call_fallible(&chat(1), "set_manager", grant(1, false))
        .await;
    assert!(steps_down.is_err(), "the last manager stays");
    let () = conductors[1]
        .call(&chat(1), "set_manager", grant(2, true))
        .await;
    settle(cells).await;
    let () = conductors[2]
        .call(&chat(2), "rename_group", "Cat's crew")
        .await;
    let () = conductors[1]
        .call(&chat(1), "set_manager", grant(1, false))
        .await;
    settle(cells).await;
    let dan: ChatSummary = conductors[3].call(&chat(3), "summary", ()).await;
    assert_eq!(dan.managers, vec![agent(2)], "Bob stepped down");
    assert_eq!(dan.title.as_deref(), Some("Cat's crew"));
    let bobs: Result<(), _> = conductors[1]
        .call_fallible(&chat(1), "rename_group", "Bob's")
        .await;
    assert!(bobs.is_err(), "Bob no longer manages it");
}
