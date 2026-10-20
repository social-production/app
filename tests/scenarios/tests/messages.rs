//! M4c direct messages and group chats (ADR 0043): the creator invites people, invitations
//! reach them encrypted through the inbox, only invited people write, and read marks are
//! private.

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
struct InviteInput {
    to: AgentPubKey,
    seed: String,
}

#[derive(Deserialize, Debug)]
struct Invitation {
    from: AgentPubKey,
    seed: String,
}

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
    preview: String,
}

#[derive(Deserialize, Debug)]
struct ChatSummary {
    kind: String,
    creator: AgentPubKey,
    members: Vec<AgentPubKey>,
    title: Option<String>,
    last: Option<LastMessage>,
    unread: u32,
}

#[derive(Deserialize, Debug)]
struct LastMessage {
    author: AgentPubKey,
    preview: String,
}

fn post(body: &str, reply_to: Option<&ActionHash>) -> PostInput {
    PostInput {
        body: body.into(),
        reply_to: reply_to.cloned(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn invited_people_talk_and_others_cannot() {
    let mut conductors = SweetConductorBatch::standard(3).await;
    let (cells, seed) = chat_network(&mut conductors, "group").await;
    let agent = |i: usize| cells[i].1.agent_pubkey().clone();
    let inbox = |i: usize| cells[i].0.zome("messages");
    let chat = |i: usize| cells[i].1.zome("messages");
    start_group(&conductors, &cells, &seed).await;

    let inboxes: Vec<SweetCell> = cells.iter().map(|(i, _)| i.clone()).collect();
    let chats: Vec<SweetCell> = cells.iter().map(|(_, c)| c.clone()).collect();
    await_consistency(&inboxes)
        .await
        .expect("inbox consistency");
    await_consistency(&chats).await.expect("chat consistency");

    let bobs: Vec<Invitation> = conductors[1].call(&inbox(1), "inbox", ()).await;
    assert_eq!(bobs.len(), 1);
    assert_eq!(bobs[0].seed, seed, "Bob can open his invitation");
    assert_eq!(bobs[0].from, agent(0));
    let cats: Vec<Invitation> = conductors[2].call(&inbox(2), "inbox", ()).await;
    assert!(cats.is_empty(), "nobody else sees an invitation");

    let hello: ActionHash = conductors[0]
        .call(&chat(0), "post_message", post("Hello Bob", None))
        .await;
    await_consistency(&chats).await.expect("chat consistency");
    let reply: ActionHash = conductors[1]
        .call(&chat(1), "post_message", post("Hi Ann", Some(&hello)))
        .await;
    let crasher: Result<ActionHash, _> = conductors[2]
        .call_fallible(&chat(2), "post_message", post("Let me in", None))
        .await;
    assert!(crasher.is_err(), "people who weren't invited can't write");
    await_consistency(&chats).await.expect("chat consistency");

    let seen: Vec<MessageView> = conductors[0].call(&chat(0), "list_messages", ()).await;
    let ids: Vec<&ActionHash> = seen.iter().map(|m| &m.id).collect();
    assert_eq!(ids, vec![&hello, &reply], "oldest first");
    assert_eq!(seen[1].author, agent(1));
    assert_eq!(seen[1].body, "Hi Ann");
    let quote = seen[1].reply_to.as_ref().expect("a reply quotes");
    assert_eq!((&quote.id, quote.preview.as_str()), (&hello, "Hello Bob"));

    Box::pin(summaries_and_deletes(&conductors, &cells, &hello, &reply)).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn direct_chats_are_for_two() {
    let mut conductors = SweetConductorBatch::standard(3).await;
    let (cells, seed) = chat_network(&mut conductors, "direct").await;
    let agent = |i: usize| cells[i].1.agent_pubkey().clone();
    let chat = cells[0].1.zome("messages");

    let crowd = StartInput {
        members: vec![agent(1), agent(2)],
        title: None,
    };
    let crowded: Result<(), _> = conductors[0]
        .call_fallible(&chat, "start_chat", crowd)
        .await;
    assert!(crowded.is_err(), "a direct chat is between two people");
    let pair = StartInput {
        members: vec![agent(1)],
        title: Some("ignored".into()),
    };
    let () = conductors[0].call(&chat, "start_chat", pair).await;
    let summary: ChatSummary = conductors[0].call(&chat, "summary", ()).await;
    assert_eq!(summary.kind, "direct");
    assert_eq!(summary.title, None, "direct chats have no name");
    assert_eq!(summary.members.len(), 2);

    let wrong = InviteInput {
        to: agent(1),
        seed: "not-a-chat".into(),
    };
    let sent: Result<(), _> = conductors[0]
        .call_fallible(&cells[0].0.zome("messages"), "send_invite", wrong)
        .await;
    assert!(sent.is_err(), "invitations carry a chat's seed");
    let in_chat = InviteInput { to: agent(1), seed };
    let misplaced: Result<(), _> = conductors[0]
        .call_fallible(&chat, "send_invite", in_chat)
        .await;
    assert!(misplaced.is_err(), "invitations go through the inbox");
}

/** Ann's summary of the group, private read marks, and deleting your own message only. */
async fn summaries_and_deletes(
    conductors: &SweetConductorBatch,
    cells: &[(SweetCell, SweetCell)],
    hello: &ActionHash,
    reply: &ActionHash,
) {
    let agent = |i: usize| cells[i].1.agent_pubkey().clone();
    let chat = |i: usize| cells[i].1.zome("messages");
    let ann: ChatSummary = conductors[0].call(&chat(0), "summary", ()).await;
    assert_eq!(ann.kind, "group");
    assert_eq!(ann.creator, agent(0));
    assert_eq!(
        ann.members,
        vec![agent(0), agent(1)],
        "invited once, creator first"
    );
    assert_eq!(ann.title.as_deref(), Some("Garden crew"));
    let last = ann.last.expect("a last message");
    assert_eq!((last.author, last.preview.as_str()), (agent(1), "Hi Ann"));
    assert_eq!(ann.unread, 1, "Bob's reply is unread");
    let () = conductors[0].call(&chat(0), "mark_read", ()).await;
    let ann: ChatSummary = conductors[0].call(&chat(0), "summary", ()).await;
    assert_eq!(ann.unread, 0, "read marks count at once");
    let bob: ChatSummary = conductors[1].call(&chat(1), "summary", ()).await;
    assert_eq!(bob.unread, 1, "read marks are each person's own");

    let () = conductors[1]
        .call(&chat(1), "delete_message", reply.clone())
        .await;
    let others: Result<(), _> = conductors[1]
        .call_fallible(&chat(1), "delete_message", hello.clone())
        .await;
    assert!(others.is_err(), "you delete only your own messages");
    let left: Vec<MessageView> = conductors[1].call(&chat(1), "list_messages", ()).await;
    assert_eq!(left.len(), 1, "your deletion shows at once");
}

/** Ann starts a group with Bob (Cat is on the network but not invited) and sends the seed. */
async fn start_group(
    conductors: &SweetConductorBatch,
    cells: &[(SweetCell, SweetCell)],
    seed: &str,
) {
    let agent = |i: usize| cells[i].1.agent_pubkey().clone();
    let inbox = |i: usize| cells[i].0.zome("messages");
    let chat = |i: usize| cells[i].1.zome("messages");
    let start = StartInput {
        members: vec![agent(1), agent(1)],
        title: Some("  Garden crew ".into()),
    };
    let () = conductors[0].call(&chat(0), "start_chat", start).await;
    for to in [agent(0), agent(1)] {
        let invite = InviteInput {
            to,
            seed: seed.to_string(),
        };
        let () = conductors[0].call(&inbox(0), "send_invite", invite).await;
    }
    let mine: Vec<Invitation> = conductors[0].call(&inbox(0), "inbox", ()).await;
    assert_eq!(mine.len(), 1, "your own chats are in your inbox at once");

    let bobs_start: Result<(), _> = conductors[1]
        .call_fallible(
            &chat(1),
            "start_chat",
            StartInput {
                members: vec![agent(2)],
                title: None,
            },
        )
        .await;
    assert!(bobs_start.is_err(), "only the creator invites");
}
