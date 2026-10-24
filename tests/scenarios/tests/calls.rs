//! Call lines in direct chats (web's `log_call`, ADR 0051): a finished call leaves one line,
//! whichever phone reports it first; a second report of the same call is a no-op.

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

#[derive(Serialize, Debug)]
struct CallInput {
    call_id: String,
    outcome: String,
    seconds: u32,
}

#[derive(Deserialize, Debug)]
struct CallLine {
    outcome: String,
    seconds: u32,
}

#[derive(Deserialize, Debug)]
struct MessageView {
    author: AgentPubKey,
    kind: String,
    call: Option<CallLine>,
}

#[derive(Deserialize, Debug)]
struct LastMessage {
    preview: String,
    kind: String,
}

#[derive(Deserialize, Debug)]
struct ChatSummary {
    last: Option<LastMessage>,
    unread: u32,
}

fn call(id: &str, outcome: &str, seconds: u32) -> CallInput {
    CallInput {
        call_id: id.into(),
        outcome: outcome.into(),
        seconds,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_finished_call_leaves_one_line() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let (cells, _seed) = chat_network(&mut conductors, "direct").await;
    let chat = |i: usize| cells[i].1.zome("messages");
    let chats: Vec<SweetCell> = cells.iter().map(|(_, c)| c.clone()).collect();
    let bob = cells[1].1.agent_pubkey().clone();
    let start = StartInput {
        members: vec![bob.clone()],
        title: None,
    };
    let () = conductors[0].call(&chat(0), "start_chat", start).await;
    let hello = PostInput {
        body: "Hello Bob".into(),
        reply_to: None,
    };
    let _: ActionHash = conductors[0].call(&chat(0), "post_message", hello).await;
    await_consistency(&chats).await.expect("chat consistency");

    // Both phones report the call that connected; the first report is the line.
    let () = conductors[0]
        .call(&chat(0), "log_call", call("c1", "completed", 252))
        .await;
    await_consistency(&chats).await.expect("chat consistency");
    let () = conductors[1]
        .call(&chat(1), "log_call", call("c1", "completed", 251))
        .await;

    let bad: Result<(), _> = conductors[1]
        .call_fallible(&chat(1), "log_call", call("c2", "missed", 5))
        .await;
    assert!(bad.is_err(), "only a completed call has a length");
    let unknown: Result<(), _> = conductors[1]
        .call_fallible(&chat(1), "log_call", call("c2", "busy", 0))
        .await;
    assert!(unknown.is_err(), "web's outcomes only");
    let () = conductors[1]
        .call(&chat(1), "log_call", call("c2", "missed", 0))
        .await;
    await_consistency(&chats).await.expect("chat consistency");

    let shown: Vec<MessageView> = conductors[0].call(&chat(0), "list_messages", ()).await;
    let calls: Vec<&MessageView> = shown.iter().filter(|m| m.kind == "call").collect();
    assert_eq!(calls.len(), 2, "one line per call: {shown:?}");
    let first = calls[0].call.as_ref().expect("call line");
    assert_eq!((first.outcome.as_str(), first.seconds), ("completed", 252));
    let missed = calls[1].call.as_ref().expect("call line");
    assert_eq!(missed.outcome, "missed");
    assert_eq!(calls[1].author, bob, "Bob logged the missed call");

    let summary: ChatSummary = conductors[0].call(&chat(0), "summary", ()).await;
    let last = summary.last.expect("a last line");
    assert_eq!((last.kind.as_str(), last.preview.as_str()), ("call", ""));
    assert_eq!(summary.unread, 1, "Bob's missed call is unread for Ann");
    common::shut_down(conductors).await;
}
