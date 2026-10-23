//! Read receipts in direct chats (web's `peer_read_at`: ✓ sent, ✓✓ read): reading a direct
//! chat tells the other person how far you have read; groups have no receipts.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetCell, SweetConductorBatch};
use serde::{Deserialize, Serialize};

mod common;

use common::chats::chat_network;

/// Mirrors the messages zome's inputs (integrity crates can't share one binary).
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

#[derive(Deserialize, Debug)]
struct MessageView {
    id: ActionHash,
    created_at: Timestamp,
}

fn post(body: &str) -> PostInput {
    PostInput {
        body: body.into(),
        reply_to: None,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn direct_chats_show_how_far_the_other_person_read() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let (cells, _seed) = chat_network(&mut conductors, "direct").await;
    let chat = |i: usize| cells[i].1.zome("messages");
    let chats: Vec<SweetCell> = cells.iter().map(|(_, c)| c.clone()).collect();
    let start = StartInput {
        members: vec![cells[1].1.agent_pubkey().clone()],
        title: None,
    };
    let () = conductors[0].call(&chat(0), "start_chat", start).await;
    let hello: ActionHash = conductors[0]
        .call(&chat(0), "post_message", post("Hello Bob"))
        .await;
    await_consistency(&chats).await.expect("chat consistency");

    let before: Option<Timestamp> = conductors[0].call(&chat(0), "peer_read_at", ()).await;
    assert_eq!(before, None, "Bob hasn't read anything yet: one tick");

    let () = conductors[1].call(&chat(1), "mark_read", ()).await;
    await_consistency(&chats).await.expect("chat consistency");
    let read: Option<Timestamp> = conductors[0].call(&chat(0), "peer_read_at", ()).await;
    let shown: Vec<MessageView> = conductors[0].call(&chat(0), "list_messages", ()).await;
    let sent = shown
        .iter()
        .find(|m| m.id == hello)
        .expect("Ann's message")
        .created_at;
    let read = read.expect("Bob's receipt reaches Ann");
    assert!(read >= sent, "Bob read Ann's message: two ticks");

    let bobs: Option<Timestamp> = conductors[1].call(&chat(1), "peer_read_at", ()).await;
    assert_eq!(bobs, None, "your own receipt isn't the other person's");
    common::shut_down(conductors).await;
}
