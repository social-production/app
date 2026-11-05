//! Reports on direct and group chat messages (ADR 0075, web's `message` target): in a 1:1
//! chat the report's own yes removes the message; in a group the other members decide with
//! web's quorum. Declined invites are private entries in the inbox (ADR 0061).

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetCell, SweetConductorBatch, SweetZome};
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
struct ReportInput {
    message: ActionHash,
    reason: String,
    description: String,
}

#[derive(Serialize, Debug)]
struct VoteInput {
    report: ActionHash,
    yes: bool,
}

#[derive(Deserialize, Debug)]
struct ReportView {
    id: ActionHash,
    resolution: String,
    yes: u32,
    audience: u32,
    delete_quorum: u32,
}

#[derive(Deserialize, Debug)]
struct Moderation {
    state: String,
    report: Option<ReportView>,
}

#[derive(Deserialize, Debug)]
struct MessageModeration {
    id: ActionHash,
    moderation: Moderation,
}

type Cells = Vec<(SweetCell, SweetCell)>;

fn post(body: &str) -> PostInput {
    PostInput {
        body: body.into(),
        reply_to: None,
    }
}

fn spam(message: &ActionHash) -> ReportInput {
    ReportInput {
        message: message.clone(),
        reason: "spam".into(),
        description: "Selling things".into(),
    }
}

async fn settle(cells: &Cells) {
    let chats: Vec<SweetCell> = cells.iter().map(|(_, c)| c.clone()).collect();
    await_consistency(&chats).await.expect("chat consistency");
}

/// Start the chat as the first agent with everyone else, post one message, settle.
async fn chat_with_message(
    conductors: &mut SweetConductorBatch,
    kind: &str,
) -> (Cells, String, ActionHash) {
    let (cells, seed) = chat_network(conductors, kind).await;
    let members = cells[1..]
        .iter()
        .map(|(_, c)| c.agent_pubkey().clone())
        .collect();
    let start = StartInput {
        members,
        title: (kind == "group").then(|| "Crew".into()),
    };
    let zome = cells[0].1.zome("messages");
    let () = conductors[0].call(&zome, "start_chat", start).await;
    settle(&cells).await;
    let hello: ActionHash = conductors[0]
        .call(&zome, "post_message", post("Buy my stuff"))
        .await;
    settle(&cells).await;
    (cells, seed, hello)
}

async fn moderation_of(c: &SweetConductorBatch, i: usize, z: &SweetZome, m: &ActionHash) -> String {
    let found: Vec<MessageModeration> = c[i].call(z, "message_moderation", vec![m.clone()]).await;
    found
        .into_iter()
        .find(|f| &f.id == m)
        .map_or_else(|| "visible".into(), |f| f.moderation.state)
}

#[tokio::test(flavor = "multi_thread")]
async fn one_report_removes_a_message_in_a_direct_chat_and_declines_stay_private() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let (cells, seed, hello) = chat_with_message(&mut conductors, "direct").await;
    let chat = |i: usize| cells[i].1.zome("messages");

    let own: Result<Moderation, _> = conductors[0]
        .call_fallible(&chat(0), "report_message", spam(&hello))
        .await;
    assert!(own.is_err(), "you can't report yourself");
    let bobs: Moderation = conductors[1]
        .call(&chat(1), "report_message", spam(&hello))
        .await;
    let report = bobs.report.expect("Bob's report");
    assert_eq!(bobs.state, "removed", "web's 1:1 path: one vote decides");
    assert_eq!(
        (report.yes, report.audience, report.delete_quorum),
        (1, 1, 1)
    );
    settle(&cells).await;
    assert_eq!(
        moderation_of(&conductors, 0, &chat(0), &hello).await,
        "removed"
    );
    let again: Result<Moderation, _> = conductors[1]
        .call_fallible(
            &chat(1),
            "vote_report",
            VoteInput {
                report: report.id,
                yes: false,
            },
        )
        .await;
    assert!(again.is_err(), "a removed report takes no more votes");

    // Declines (ADR 0061): only in the inbox, and only the decliner sees them.
    let inbox = |i: usize| cells[i].0.zome("messages");
    let in_chat: Result<(), _> = conductors[1]
        .call_fallible(&chat(1), "decline_invite", seed.clone())
        .await;
    assert!(in_chat.is_err(), "declines live in the inbox");
    let () = conductors[1]
        .call(&inbox(1), "decline_invite", seed.clone())
        .await;
    let mine: Vec<String> = conductors[1].call(&inbox(1), "declined_invites", ()).await;
    assert_eq!(mine, vec![seed]);
    let anns: Vec<String> = conductors[0].call(&inbox(0), "declined_invites", ()).await;
    assert!(anns.is_empty(), "a decline stays on its author's chain");
    common::shut_down(conductors).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_group_removes_a_message_once_three_others_vote_yes() {
    let mut conductors = SweetConductorBatch::standard(4).await;
    let (cells, _, hello) = chat_with_message(&mut conductors, "group").await;
    let chat = |i: usize| cells[i].1.zome("messages");

    let opened: Moderation = conductors[1]
        .call(&chat(1), "report_message", spam(&hello))
        .await;
    assert_eq!(opened.state, "under_review");
    let report = opened.report.expect("Bob's report");
    assert_eq!((report.audience, report.delete_quorum), (3, 3));
    settle(&cells).await;
    let vote = |yes| VoteInput {
        report: report.id.clone(),
        yes,
    };
    let anns: Result<Moderation, _> = conductors[0]
        .call_fallible(&chat(0), "vote_report", vote(false))
        .await;
    assert!(
        anns.is_err(),
        "the author doesn't vote on their own message"
    );
    let cats: Moderation = conductors[2]
        .call(&chat(2), "vote_report", vote(true))
        .await;
    assert_eq!(cats.state, "under_review");
    settle(&cells).await;
    let dans: Moderation = conductors[3]
        .call(&chat(3), "vote_report", vote(true))
        .await;
    assert_eq!(dans.state, "removed", "three yes of three others");
    assert_eq!(
        dans.report.map(|r| r.resolution).as_deref(),
        Some("removed")
    );
    settle(&cells).await;
    assert_eq!(
        moderation_of(&conductors, 0, &chat(0), &hello).await,
        "removed"
    );
    common::shut_down(conductors).await;
}
