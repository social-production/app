//! ADR 0109: a channel's or community's picture and cover are directory `ScopeImage` records
//! named by its charter and changed by its own votes; and on a #platform project any Full
//! member may vote, members or not.

use holochain::prelude::*;
use serde::{Deserialize, Serialize};

mod common;

use common::net::{charter, Charter, Net};

/// Mirrors `project_integrity::Charter` with ADR 0109's picture and cover.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ScopeCharter {
    title: String,
    description: String,
    mode: String,
    subtype: String,
    location_label: String,
    phase: String,
    tags: Vec<String>,
    picture: Option<EntryHash>,
    cover: Option<EntryHash>,
}

fn with_art(c: Charter, picture: Option<EntryHash>) -> ScopeCharter {
    ScopeCharter {
        title: c.title,
        description: c.description,
        mode: c.mode,
        subtype: c.subtype,
        location_label: c.location_label,
        phase: c.phase,
        tags: c.tags,
        picture,
        cover: None,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct ScopeImage {
    mime: String,
    #[serde(with = "serde_bytes")]
    bytes: Vec<u8>,
}

#[derive(Serialize, Debug)]
enum ImageChange {
    Set(EntryHash),
}

#[derive(Serialize, Debug)]
enum ProposalKind {
    Edit {
        title: Option<String>,
        description: Option<String>,
        location: Option<String>,
        cover: Option<ImageChange>,
    },
}

#[derive(Serialize, Debug)]
enum VoteChoice {
    Yes,
}

#[derive(Serialize, Debug)]
struct VoteInput {
    proposal: ActionHash,
    choice: VoteChoice,
}

#[derive(Deserialize, Debug)]
struct ProjectView {
    charter: Option<ScopeCharter>,
}

async fn put_image(net: &Net, i: usize, byte: u8) -> EntryHash {
    let image = ScopeImage {
        mime: "image/png".into(),
        bytes: vec![byte; 64],
    };
    let zome = net.cells[i].0.zome("identity");
    Box::pin(net.conductors[i].call(&zome, "put_scope_image", image)).await
}

async fn charter_of(net: &Net, i: usize) -> ScopeCharter {
    let view: Option<ProjectView> = net.call(i, "project", "get_project", ()).await;
    view.and_then(|v| v.charter).expect("the charter")
}

async fn vote(net: &Net, i: usize, proposal: &ActionHash) -> Result<ActionHash, String> {
    let input = VoteInput {
        proposal: proposal.clone(),
        choice: VoteChoice::Yes,
    };
    net.try_call(i, "governance", "vote", input).await
}

fn edit(description: Option<&str>, cover: Option<EntryHash>) -> ProposalKind {
    ProposalKind::Edit {
        title: None,
        description: description.map(Into::into),
        location: None,
        cover: cover.map(ImageChange::Set),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_channel_takes_a_picture_and_votes_on_a_new_cover() {
    // Agents 0–2 are Full; agent 0 founds #seeds with a picture (integrity takes any hash; the
    // image is the directory's).
    let picture = EntryHash::from_raw_36(vec![7; 36]);
    let seeds = ScopeCharter {
        title: "Seeds".into(),
        subtype: "open".into(),
        ..with_art(charter("channel"), Some(picture.clone()))
    };
    let net = Net::with_charter(3, 3, "pictures", seeds).await;
    assert_eq!(charter_of(&net, 1).await.picture, Some(picture.clone()));
    let productive = with_art(charter("productive"), Some(picture));
    let refused: Result<ActionHash, String> = net
        .try_call(0, "project", "create_charter", productive)
        .await;
    assert!(refused.is_err(), "only scopes have pictures: {refused:?}");

    let cover = put_image(&net, 1, 2).await;
    let proposal: ActionHash = net
        .call(1, "governance", "propose", edit(None, Some(cover.clone())))
        .await;
    net.settle().await;
    vote(&net, 2, &proposal).await.expect("a member votes");
    // A third yes, in case three members need three; once decided, it is refused.
    let _ = vote(&net, 0, &proposal).await;
    net.settle().await;
    assert_eq!(charter_of(&net, 0).await.cover, Some(cover.clone()));
    let zome = net.cells[2].0.zome("identity");
    let read: Option<ScopeImage> =
        Box::pin(net.conductors[2].call(&zome, "scope_image", cover)).await;
    assert_eq!(read.map(|i| i.bytes), Some(vec![2; 64]));
    net.shut_down().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn any_full_member_votes_on_a_platform_project() {
    // Agents 0–2 are Full, agent 3 is New; all join, then 2 and 3 leave. Two members remain,
    // so the edit needs two yes votes: the proposer's and a Full non-member's.
    let platform = Charter {
        tags: vec!["#platform".into()],
        ..charter("productive")
    };
    let net = Net::with_charter(4, 3, "platform-votes", platform).await;
    for i in [2, 3] {
        let _: () = net.call(i, "project", "leave", ()).await;
    }
    net.settle().await;
    let proposal: ActionHash = net
        .call(
            0,
            "governance",
            "propose",
            edit(Some("For the whole network."), None),
        )
        .await;
    net.settle().await;
    let new = vote(&net, 3, &proposal).await;
    assert!(new.is_err(), "a New non-member can't vote: {new:?}");
    vote(&net, 2, &proposal)
        .await
        .expect("a Full non-member votes");
    net.settle().await;
    assert_eq!(
        charter_of(&net, 1).await.description,
        "For the whole network."
    );
    net.shut_down().await;
}
