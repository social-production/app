//! M4c group options, edits and pins (ADR 0045): the manager renames, adds and removes; a
//! leaving manager hands over; people edit their own messages; pins follow web's rights.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetCell, SweetConductorBatch};
use serde::{Deserialize, Serialize};

mod common;

use common::chats::chat_network;

/// Mirrors the messages zome's inputs and views (integrity crates can't share one binary).
#[derive(Serialize, Debug)]
struct StartInput {
    members: Vec<AgentPubKey>,
    title: Option<String>,
}

#[derive(Serialize, Debug)]
struct PostInput {
    body: String,
    reply_to: Option<ActionHash>,
}

#[derive(Serialize, Debug)]
struct EditInput {
    id: ActionHash,
    body: String,
}

#[derive(Serialize, Debug)]
struct PinInput {
    id: ActionHash,
    pinned: bool,
}

#[derive(Serialize, Debug, Default)]
struct PrefsInput {
    pinned: Option<bool>,
    muted: Option<bool>,
    hidden: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct MessageView {
    author: AgentPubKey,
    body: String,
    kind: String,
    edited: bool,
    pinned: bool,
}

#[derive(Deserialize, Debug)]
struct PinView {
    message: ActionHash,
    preview: String,
}

#[allow(clippy::struct_excessive_bools)] // mirrors the zome's view
#[derive(Deserialize, Debug)]
struct ChatSummary {
    manager: Option<AgentPubKey>,
    members: Vec<AgentPubKey>,
    title: Option<String>,
    member: bool,
    can_pin: bool,
    pinned: bool,
    muted: bool,
    hidden: bool,
}

fn post(body: &str) -> PostInput {
    PostInput {
        body: body.into(),
        reply_to: None,
    }
}

type Cells = Vec<(SweetCell, SweetCell)>;

async fn settle(cells: &Cells) {
    let chats: Vec<SweetCell> = cells.iter().map(|(_, c)| c.clone()).collect();
    await_consistency(&chats).await.expect("chat consistency");
}

async fn summary(conductors: &SweetConductorBatch, cells: &Cells, i: usize) -> ChatSummary {
    conductors[i]
        .call(&cells[i].1.zome("messages"), "summary", ())
        .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_manager_runs_the_group_and_hands_it_over() {
    let mut conductors = SweetConductorBatch::standard(4).await;
    let (cells, _) = chat_network(&mut conductors, "group").await;
    let agent = |i: usize| cells[i].1.agent_pubkey().clone();
    let chat = |i: usize| cells[i].1.zome("messages");
    let start = StartInput {
        members: vec![agent(1), agent(2)],
        title: Some("Crew".into()),
    };
    let () = conductors[0].call(&chat(0), "start_chat", start).await;
    settle(&cells).await;

    let bobs: Result<(), _> = conductors[1]
        .call_fallible(&chat(1), "add_member", agent(3))
        .await;
    assert!(bobs.is_err(), "only the manager adds people");
    let renamed: Result<(), _> = conductors[1]
        .call_fallible(&chat(1), "rename_group", "Mine")
        .await;
    assert!(renamed.is_err(), "only the manager renames");
    let () = conductors[0]
        .call(&chat(0), "rename_group", " New crew ")
        .await;
    let () = conductors[0].call(&chat(0), "add_member", agent(3)).await;
    let () = conductors[0]
        .call(&chat(0), "remove_member", agent(2))
        .await;
    let ann = summary(&conductors, &cells, 0).await;
    assert_eq!(ann.title.as_deref(), Some("New crew"));
    assert_eq!(ann.members, vec![agent(0), agent(1), agent(3)]);
    settle(&cells).await;
    let cat = summary(&conductors, &cells, 2).await;
    assert!(!cat.member, "Cat was removed");
    let cats: Result<ActionHash, _> = conductors[2]
        .call_fallible(&chat(2), "post_message", post("Still here?"))
        .await;
    assert!(cats.is_err(), "removed people don't write");
    let hello: ActionHash = conductors[1]
        .call(&chat(1), "post_message", post("Hello"))
        .await;
    let bobs_pin: Result<(), _> = conductors[1]
        .call_fallible(
            &chat(1),
            "pin_message",
            PinInput {
                id: hello,
                pinned: true,
            },
        )
        .await;
    assert!(bobs_pin.is_err(), "in a group only the manager pins");

    Box::pin(handover(&conductors, &cells)).await;
    common::shut_down(&mut conductors).await;
}

/** Ann (the manager) leaves: Bob takes over, renames, and adds Cat back. */
async fn handover(conductors: &SweetConductorBatch, cells: &Cells) {
    let agent = |i: usize| cells[i].1.agent_pubkey().clone();
    let chat = |i: usize| cells[i].1.zome("messages");
    let () = conductors[0].call(&chat(0), "leave", ()).await;
    settle(cells).await;
    let bob = summary(conductors, cells, 1).await;
    assert_eq!(
        bob.manager,
        Some(agent(1)),
        "the longest-standing member takes over"
    );
    assert_eq!(bob.members, vec![agent(1), agent(3)]);
    assert!(bob.can_pin);
    assert!(!summary(conductors, cells, 0).await.member, "Ann left");
    let seen: Vec<MessageView> = conductors[3].call(&chat(3), "list_messages", ()).await;
    let line = seen.last().expect("a system line");
    assert_eq!((line.kind.as_str(), &line.author), ("system", &agent(0)));

    let () = conductors[1]
        .call(&chat(1), "rename_group", "Bob's crew")
        .await;
    let anns: Result<(), _> = conductors[0]
        .call_fallible(&chat(0), "rename_group", "Back")
        .await;
    assert!(anns.is_err(), "Ann no longer manages it");
    let () = conductors[1].call(&chat(1), "add_member", agent(2)).await;
    settle(cells).await;
    let _: ActionHash = conductors[2]
        .call(&chat(2), "post_message", post("Back again"))
        .await;
    let cat = summary(conductors, cells, 2).await;
    assert!(cat.member && cat.title.as_deref() == Some("Bob's crew"));
}

#[tokio::test(flavor = "multi_thread")]
async fn edits_pins_and_list_settings() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let (cells, _) = chat_network(&mut conductors, "direct").await;
    let agent = |i: usize| cells[i].1.agent_pubkey().clone();
    let chat = |i: usize| cells[i].1.zome("messages");
    let start = StartInput {
        members: vec![agent(1)],
        title: None,
    };
    let () = conductors[0].call(&chat(0), "start_chat", start).await;
    let first: ActionHash = conductors[0]
        .call(&chat(0), "post_message", post("Helo"))
        .await;
    settle(&cells).await;

    let edit = |id: &ActionHash, body: &str| EditInput {
        id: id.clone(),
        body: body.into(),
    };
    let bobs: Result<(), _> = conductors[1]
        .call_fallible(&chat(1), "edit_message", edit(&first, "Hi"))
        .await;
    assert!(bobs.is_err(), "people edit only their own messages");
    let blank: Result<(), _> = conductors[0]
        .call_fallible(&chat(0), "edit_message", edit(&first, "  "))
        .await;
    assert!(blank.is_err());
    let () = conductors[0]
        .call(&chat(0), "edit_message", edit(&first, "Hello"))
        .await;
    settle(&cells).await;
    let seen: Vec<MessageView> = conductors[1].call(&chat(1), "list_messages", ()).await;
    assert_eq!((seen[0].body.as_str(), seen[0].edited), ("Hello", true));

    Box::pin(pins(&conductors, &cells, &first)).await;
    let left: Result<(), _> = conductors[0].call_fallible(&chat(0), "leave", ()).await;
    assert!(left.is_err(), "direct chats are deleted, not left");
    Box::pin(list_settings(&conductors, &cells)).await;
    common::shut_down(&mut conductors).await;
}

/** Pin, mute and delete (hide) are your own; a new message brings a hidden chat back. */
async fn list_settings(conductors: &SweetConductorBatch, cells: &Cells) {
    let chat = |i: usize| cells[i].1.zome("messages");
    let set = PrefsInput {
        pinned: Some(true),
        muted: Some(true),
        ..PrefsInput::default()
    };
    let () = conductors[0].call(&chat(0), "set_prefs", set).await;
    let hide = PrefsInput {
        hidden: Some(true),
        ..PrefsInput::default()
    };
    let () = conductors[0].call(&chat(0), "set_prefs", hide).await;
    let ann = summary(conductors, cells, 0).await;
    assert!(ann.pinned && ann.muted && ann.hidden);
    let bob = summary(conductors, cells, 1).await;
    assert!(
        !bob.pinned && !bob.muted && !bob.hidden,
        "settings are each person's own"
    );
    let _: ActionHash = conductors[1]
        .call(&chat(1), "post_message", post("Still there?"))
        .await;
    settle(cells).await;
    assert!(
        !summary(conductors, cells, 0).await.hidden,
        "a new message unhides it"
    );
}

/** Either person pins in a direct chat, up to 5; either unpins. */
async fn pins(conductors: &SweetConductorBatch, cells: &Cells, first: &ActionHash) {
    let agent = |i: usize| cells[i].1.agent_pubkey().clone();
    let chat = |i: usize| cells[i].1.zome("messages");
    let pin = |id: &ActionHash, pinned: bool| PinInput {
        id: id.clone(),
        pinned,
    };
    let () = conductors[1]
        .call(&chat(1), "pin_message", pin(first, true))
        .await;
    let mut ids = vec![first.clone()];
    for n in 0..5 {
        let id: ActionHash = conductors[1]
            .call(&chat(1), "post_message", post(&format!("m{n}")))
            .await;
        ids.push(id);
    }
    for id in &ids[1..5] {
        let () = conductors[1]
            .call(&chat(1), "pin_message", pin(id, true))
            .await;
    }
    let sixth: Result<(), _> = conductors[1]
        .call_fallible(&chat(1), "pin_message", pin(&ids[5], true))
        .await;
    assert!(sixth.is_err(), "at most 5 pins");
    settle(cells).await;
    let pins: Vec<PinView> = conductors[0].call(&chat(0), "pins", ()).await;
    assert_eq!(pins.len(), 5);
    assert_eq!(
        (&pins[0].message, pins[0].preview.as_str()),
        (first, "Hello")
    );
    let () = conductors[0]
        .call(&chat(0), "pin_message", pin(first, false))
        .await;
    let seen: Vec<MessageView> = conductors[0].call(&chat(0), "list_messages", ()).await;
    assert!(
        !seen[0].pinned && seen[1].pinned,
        "either person unpins in a direct chat"
    );
    assert_eq!(seen[1].author, agent(1));
}
