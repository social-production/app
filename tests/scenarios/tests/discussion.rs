//! M4a project chat (ADR 0038): anyone in the project's network posts and replies; only the
//! author deletes; messages are never edited.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch};
use serde::{Deserialize, Serialize};

mod common;

/// Mirrors `discussion::PostInput` (integrity crates can't share one test binary).
#[derive(Serialize, Debug)]
struct PostInput {
    body: String,
    reply_to: Option<ActionHash>,
}

#[derive(Deserialize, Debug)]
struct MessageView {
    id: ActionHash,
    author: AgentPubKey,
    body: String,
    reply_to: Option<ReplyRef>,
}

#[derive(Deserialize, Debug)]
struct ReplyRef {
    id: ActionHash,
    author: Option<AgentPubKey>,
    preview: String,
    deleted: bool,
}

#[derive(Serialize, Debug)]
struct Charter {
    title: String,
    description: String,
    mode: String,
    subtype: String,
    location_label: String,
    phase: String,
    tags: Vec<String>,
}

fn post(body: &str, reply_to: Option<&ActionHash>) -> PostInput {
    PostInput {
        body: body.into(),
        reply_to: reply_to.cloned(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn anyone_posts_replies_and_reads() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let cells = common::project_network(&mut conductors, "chat-read").await;
    let (ann, bob) = (cells[0].zome("discussion"), cells[1].zome("discussion"));

    // Bob never joined: as on web, anyone signed in may write in a project's chat.
    let hello: ActionHash = conductors[1]
        .call(&bob, "post_message", post("  Hello all  ", None))
        .await;
    let mine: Vec<MessageView> = conductors[1].call(&bob, "list_messages", ()).await;
    assert_eq!(mine.len(), 1, "your own message shows before gossip");
    assert_eq!(mine[0].body, "Hello all", "bodies are trimmed");

    await_consistency(&cells).await.expect("consistency");
    let reply: ActionHash = conductors[0]
        .call(&ann, "post_message", post("Welcome, Bob", Some(&hello)))
        .await;
    await_consistency(&cells).await.expect("consistency");

    let seen: Vec<MessageView> = conductors[1].call(&bob, "list_messages", ()).await;
    let ids: Vec<&ActionHash> = seen.iter().map(|m| &m.id).collect();
    assert_eq!(ids, vec![&hello, &reply], "oldest first");
    assert_eq!(&seen[1].author, cells[0].agent_pubkey());
    let quote = seen[1]
        .reply_to
        .as_ref()
        .expect("a reply carries its quote");
    assert_eq!(quote.id, hello);
    assert_eq!(quote.author.as_ref(), Some(cells[1].agent_pubkey()));
    assert_eq!(quote.preview, "Hello all");
    assert!(!quote.deleted);
    common::shut_down(&mut conductors).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_messages_are_rejected() {
    let mut conductors = SweetConductorBatch::standard(1).await;
    let cells = common::project_network(&mut conductors, "chat-bad").await;
    let ann = cells[0].zome("discussion");

    let blank: Result<ActionHash, _> = conductors[0]
        .call_fallible(&ann, "post_message", post("   ", None))
        .await;
    assert!(blank.is_err(), "a message needs text");
    let long: Result<ActionHash, _> = conductors[0]
        .call_fallible(&ann, "post_message", post(&"x".repeat(4_001), None))
        .await;
    assert!(long.is_err(), "a message is at most 4,000 characters");

    // Integrity: a reply must answer a chat message, not any record.
    let charter = Charter {
        title: "Garden".into(),
        description: "Growing food together.".into(),
        mode: "productive".into(),
        subtype: "standard".into(),
        location_label: String::new(),
        phase: String::new(),
        tags: Vec::new(),
    };
    let not_a_message: ActionHash = conductors[0]
        .call(&cells[0].zome("project"), "create_charter", charter)
        .await;
    let odd: Result<ActionHash, _> = conductors[0]
        .call_fallible(
            &ann,
            "post_message",
            post("Re: the charter", Some(&not_a_message)),
        )
        .await;
    assert!(odd.is_err(), "validation rejects a reply to a non-message");
    common::shut_down(&mut conductors).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn only_the_author_deletes() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let cells = common::project_network(&mut conductors, "chat-delete").await;
    let (ann, bob) = (cells[0].zome("discussion"), cells[1].zome("discussion"));

    let first: ActionHash = conductors[1]
        .call(&bob, "post_message", post("Typo here", None))
        .await;
    await_consistency(&cells).await.expect("consistency");
    let _: ActionHash = conductors[0]
        .call(&ann, "post_message", post("What typo?", Some(&first)))
        .await;

    let other: Result<(), _> = conductors[0]
        .call_fallible(&ann, "delete_message", first.clone())
        .await;
    assert!(other.is_err(), "you can't delete someone else's message");

    let (): () = conductors[1]
        .call(&bob, "delete_message", first.clone())
        .await;
    let mine: Vec<MessageView> = conductors[1].call(&bob, "list_messages", ()).await;
    assert!(
        mine.iter().all(|m| m.id != first),
        "your own deletion shows at once"
    );
    await_consistency(&cells).await.expect("consistency");

    let seen: Vec<MessageView> = conductors[0].call(&ann, "list_messages", ()).await;
    assert_eq!(seen.len(), 1, "the deleted message is gone for everyone");
    let quote = seen[0]
        .reply_to
        .as_ref()
        .expect("the reply keeps its quote");
    assert!(quote.deleted, "and the quote reads as a deleted message");
    assert!(quote.preview.is_empty());
    common::shut_down(&mut conductors).await;
}
