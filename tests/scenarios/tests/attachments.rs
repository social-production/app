//! Photos and files on direct and group messages (web's `message_attachments`, ADR 0049): the
//! sender writes each file's parts, then a message listing them; the other person reads the
//! parts back. Parts are bounded, only people in the chat write them, and a message may cite
//! only its author's own parts, as many as its size needs.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetCell, SweetConductorBatch};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

mod common;

use common::chats::chat_network;

/// `sp_attachment::CHUNK_MAX`.
const CHUNK_MAX: usize = 1024 * 1024;

/// Mirrors the messages zome's inputs (integrity crates can't share one binary).
#[derive(Serialize, Debug)]
struct StartInput {
    members: Vec<AgentPubKey>,
    title: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Attachment {
    kind: String,
    name: String,
    mime: String,
    size: u32,
    chunks: Vec<ActionHash>,
}

#[derive(Serialize, Debug)]
struct PostInput {
    body: String,
    reply_to: Option<ActionHash>,
    attachments: Vec<Attachment>,
}

#[derive(Deserialize, Debug)]
struct MessageView {
    id: ActionHash,
    body: String,
    attachments: Vec<Attachment>,
}

#[derive(Deserialize, Debug)]
struct LastMessage {
    preview: String,
}

#[derive(Deserialize, Debug)]
struct Summary {
    last: Option<LastMessage>,
}

fn attachment(
    kind: &str,
    name: &str,
    mime: &str,
    size: usize,
    chunks: &[ActionHash],
) -> Attachment {
    Attachment {
        kind: kind.into(),
        name: name.into(),
        mime: mime.into(),
        size: u32::try_from(size).expect("small"),
        chunks: chunks.to_vec(),
    }
}

fn post(attachments: Vec<Attachment>) -> PostInput {
    PostInput {
        body: String::new(),
        reply_to: None,
        attachments,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn photos_and_files_reach_the_other_person() {
    let mut conductors = SweetConductorBatch::standard(3).await;
    let (cells, _seed) = chat_network(&mut conductors, "direct").await;
    let chat = |i: usize| cells[i].1.zome("messages");
    let chats: Vec<SweetCell> = cells.iter().map(|(_, c)| c.clone()).collect();
    let start = StartInput {
        members: vec![cells[1].1.agent_pubkey().clone()],
        title: None,
    };
    let () = conductors[0].call(&chat(0), "start_chat", start).await;

    let (photo, file) = (bytes(5_000, 251), bytes(CHUNK_MAX + 10, 7));
    let (id, sent, file_parts) =
        Box::pin(send_files(&conductors, &cells[0].1, &photo, &file)).await;
    await_consistency(&chats).await.expect("chat consistency");

    let seen: Vec<MessageView> = conductors[1].call(&chat(1), "list_messages", ()).await;
    let message = seen
        .iter()
        .find(|m| m.id == id)
        .expect("Bob sees Ann's message");
    assert_eq!(message.body, "", "a photo needs no caption");
    assert_eq!(message.attachments, sent);
    let mut got = Vec::new();
    for part in &message.attachments[1].chunks {
        let bytes: Option<ByteBuf> = conductors[1]
            .call(&chat(1), "get_chunk", part.clone())
            .await;
        got.extend(bytes.expect("Bob reads the part").into_vec());
    }
    assert_eq!(got, file, "the parts make the file again");
    let summary: Summary = conductors[1].call(&chat(1), "summary", ()).await;
    assert_eq!(summary.last.expect("a last message").preview, "Photo");

    Box::pin(refusals(&conductors, &cells, &file_parts)).await;

    let () = conductors[0]
        .call(&chat(0), "delete_message", id.clone())
        .await;
    let after: Vec<MessageView> = conductors[0].call(&chat(0), "list_messages", ()).await;
    assert!(
        after.iter().all(|m| m.id != id),
        "the message and its files go"
    );
    common::shut_down(conductors).await;
}

/// `n` bytes counting up to `modulo`, so parts in the wrong order show.
fn bytes(n: usize, modulo: usize) -> Vec<u8> {
    (0..n)
        .map(|i| u8::try_from(i % modulo).expect("below 256"))
        .collect()
}

/// Ann writes a photo (one part) and a file (two parts), then a message listing both.
async fn send_files(
    conductors: &SweetConductorBatch,
    cell: &SweetCell,
    photo: &[u8],
    file: &[u8],
) -> (ActionHash, Vec<Attachment>, Vec<ActionHash>) {
    let chat = cell.zome("messages");
    let photo_part: ActionHash = conductors[0]
        .call(&chat, "upload_chunk", ByteBuf::from(photo.to_vec()))
        .await;
    let mut file_parts: Vec<ActionHash> = Vec::new();
    for part in file.chunks(CHUNK_MAX) {
        file_parts.push(
            conductors[0]
                .call(&chat, "upload_chunk", ByteBuf::from(part.to_vec()))
                .await,
        );
    }
    let sent = vec![
        attachment(
            "image",
            "garden.jpg",
            "image/jpeg",
            photo.len(),
            &[photo_part],
        ),
        attachment(
            "file",
            "plan.pdf",
            "application/pdf",
            file.len(),
            &file_parts,
        ),
    ];
    let id: ActionHash = conductors[0]
        .call(&chat, "post_message", post(sent.clone()))
        .await;
    (id, sent, file_parts)
}

async fn refusals(
    conductors: &SweetConductorBatch,
    cells: &[(SweetCell, SweetCell)],
    anns_parts: &[ActionHash],
) {
    let chat = |i: usize| cells[i].1.zome("messages");
    let huge: Result<ActionHash, _> = conductors[0]
        .call_fallible(
            &chat(0),
            "upload_chunk",
            ByteBuf::from(vec![1u8; CHUNK_MAX + 1]),
        )
        .await;
    assert!(huge.is_err(), "a part is at most 1 MB");
    let outsider: Result<ActionHash, _> = conductors[2]
        .call_fallible(&chat(2), "upload_chunk", ByteBuf::from(vec![1u8; 10]))
        .await;
    assert!(
        outsider.is_err(),
        "people outside the chat can't share files in it"
    );

    let short = attachment(
        "file",
        "plan.pdf",
        "application/pdf",
        CHUNK_MAX + 10,
        &anns_parts[..1],
    );
    let wrong: Result<ActionHash, _> = conductors[0]
        .call_fallible(&chat(0), "post_message", post(vec![short]))
        .await;
    assert!(
        wrong.is_err(),
        "a file lists as many parts as its size needs"
    );
    let video = attachment("file", "clip.mp4", "video/mp4", 10, &anns_parts[..1]);
    let refused: Result<ActionHash, _> = conductors[0]
        .call_fallible(&chat(0), "post_message", post(vec![video]))
        .await;
    assert!(refused.is_err(), "videos are not supported yet, as on web");

    let borrowed = attachment(
        "file",
        "plan.pdf",
        "application/pdf",
        CHUNK_MAX + 10,
        anns_parts,
    );
    let theft: Result<ActionHash, _> = conductors[1]
        .call_fallible(&chat(1), "post_message", post(vec![borrowed]))
        .await;
    assert!(
        theft.is_err(),
        "a message cites only its author's own parts"
    );
}
