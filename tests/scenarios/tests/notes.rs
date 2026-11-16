//! Covers and comment notes (Profiles and Comments mockups §2–3): a cover is replaced or removed
//! by its owner and read by others; a comment note lists under its author, newest first, goes
//! when the comment is deleted, and refuses blank text or a link from someone else's key.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch};
use identity_integrity::{CommentNote, Cover};
use serde::Deserialize;

mod common;

#[derive(Deserialize, Debug)]
struct NoteView {
    author: AgentPubKey,
    note: CommentNote,
}

fn cover(byte: u8) -> Cover {
    Cover {
        mime: "image/jpeg".into(),
        bytes: vec![byte; 2048],
    }
}

fn note(space: &ActionHash, message: u8, excerpt: &str) -> CommentNote {
    CommentNote {
        space: space.clone(),
        message: ActionHash::from_raw_36(vec![message; 36]),
        reply_to: None,
        excerpt: excerpt.into(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn covers_and_comment_notes() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let cells = common::directory_network(&mut conductors, 1).await;
    let (ann, bob) = (cells[0].zome("identity"), cells[1].zome("identity"));
    let ann_key = cells[0].agent_pubkey().clone();

    // Covers: set, replace, read by someone else, then removed.
    let _: Option<ActionHash> = conductors[0].call(&ann, "set_cover", Some(cover(1))).await;
    let _: Option<ActionHash> = conductors[0].call(&ann, "set_cover", Some(cover(2))).await;
    let mine: Option<Cover> = conductors[0].call(&ann, "get_cover", ann_key.clone()).await;
    assert_eq!(mine.map(|c| c.bytes[0]), Some(2), "the newest cover counts");
    let big = Cover {
        mime: "image/jpeg".into(),
        bytes: vec![0; identity_integrity::COVER_MAX_BYTES + 1],
    };
    let refused: Result<Option<ActionHash>, _> = conductors[0]
        .call_fallible(&ann, "set_cover", Some(big))
        .await;
    assert!(refused.is_err(), "a cover has a size cap");
    await_consistency(&cells).await.expect("consistency");
    let seen: Option<Cover> = conductors[1].call(&bob, "get_cover", ann_key.clone()).await;
    assert_eq!(seen.map(|c| c.bytes[0]), Some(2));

    // Comment notes: two notes, newest first, read by Bob; one taken back.
    let space = ActionHash::from_raw_36(vec![7; 36]);
    let _: ActionHash = conductors[0]
        .call(&ann, "note_comment", note(&space, 1, "Tomatoes did best"))
        .await;
    let _: ActionHash = conductors[0]
        .call(&ann, "note_comment", note(&space, 2, "And beans, again"))
        .await;
    let blank: Result<ActionHash, _> = conductors[0]
        .call_fallible(&ann, "note_comment", note(&space, 3, "  "))
        .await;
    assert!(blank.is_err(), "a note needs its opening words");
    await_consistency(&cells).await.expect("consistency");
    let notes: Vec<NoteView> = conductors[1]
        .call(&bob, "comments_of", ann_key.clone())
        .await;
    assert_eq!(
        notes
            .iter()
            .map(|n| n.note.excerpt.as_str())
            .collect::<Vec<_>>(),
        ["And beans, again", "Tomatoes did best"]
    );
    assert!(notes.iter().all(|n| n.author == ann_key));

    let () = conductors[0]
        .call(&ann, "unnote_comment", ActionHash::from_raw_36(vec![2; 36]))
        .await;
    let _: Option<ActionHash> = conductors[0].call(&ann, "set_cover", None::<Cover>).await;
    await_consistency(&cells).await.expect("consistency");
    let left: Vec<NoteView> = conductors[1]
        .call(&bob, "comments_of", ann_key.clone())
        .await;
    assert_eq!(left.len(), 1, "a deleted comment's note goes");
    let gone: Option<Cover> = conductors[1].call(&bob, "get_cover", ann_key).await;
    assert!(gone.is_none(), "a removed cover is gone");
}
