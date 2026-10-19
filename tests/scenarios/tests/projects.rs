//! M2 projects: charter, membership, signals (project DNA) and listings (directory DNA).

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch};
use serde::{Deserialize, Serialize};

mod common;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
enum Stance {
    Demand,
    Oppose,
}

/// Mirrors `project_integrity::Charter` (two integrity crates cannot link into one test binary).
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Charter {
    title: String,
    description: String,
    mode: String,
    subtype: String,
    location_label: String,
    phase: String,
    tags: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct ProjectView {
    creator: AgentPubKey,
    charter: Option<Charter>,
    members: Vec<AgentPubKey>,
    demand: u32,
    opposition: u32,
    am_member: bool,
    my_signal: Option<Stance>,
}

#[derive(Serialize, Debug)]
struct ListingInput {
    network_seed: String,
    title: String,
    summary: String,
    mode: String,
    subtype: String,
    location_label: String,
    phase: String,
    tags: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct ListingView {
    creator: AgentPubKey,
    listing: ListingFields,
}

#[derive(Deserialize, Debug)]
struct ListingFields {
    title: String,
}

#[derive(Serialize, Debug)]
struct ListingUpdate {
    id: ActionHash,
    listing: ListingInput,
}

#[derive(Deserialize, Debug)]
struct Allowance {
    remaining: Option<u32>,
}

fn charter(title: &str) -> Charter {
    Charter {
        title: title.into(),
        description: "Growing food together.".into(),
        mode: "productive".into(),
        subtype: "standard".into(),
        location_label: "Brisbane".into(),
        phase: String::new(),
        tags: vec!["food".into()],
    }
}

fn listing(creator: &AgentPubKey, seed: &str) -> ListingInput {
    ListingInput {
        network_seed: format!(
            "{}.{seed}",
            holo_hash::AgentPubKeyB64::from(creator.clone())
        ),
        title: format!("Project {seed}"),
        summary: "A project".into(),
        mode: "productive".into(),
        subtype: "standard".into(),
        location_label: String::new(),
        phase: String::new(),
        tags: Vec::new(),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn members_join_signal_and_leave() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let cells = common::project_network(&mut conductors, "garden").await;
    let (ann, bob) = (cells[0].zome("project"), cells[1].zome("project"));

    let _: ActionHash = conductors[0]
        .call(&ann, "create_charter", charter("Garden"))
        .await;
    let forged: Result<ActionHash, _> = conductors[1]
        .call_fallible(&bob, "create_charter", charter("Mine now"))
        .await;
    assert!(forged.is_err(), "only the creator writes the charter");
    let _: () = conductors[1].call(&bob, "join", ()).await;
    let _: () = conductors[1]
        .call(&bob, "set_signal", Some(Stance::Demand))
        .await;
    await_consistency(&cells).await.expect("consistency");

    let seen: ProjectView = conductors[0].call(&ann, "get_project", ()).await;
    assert_eq!(seen.creator, cells[0].agent_pubkey().clone());
    assert_eq!(seen.charter.map(|c| c.title), Some("Garden".into()));
    assert_eq!(seen.members.len(), 2);
    assert!(seen.am_member, "the creator joins on create");
    assert_eq!((seen.demand, seen.opposition), (1, 0));

    // One live signal per agent: switching replaces it; own view updates at once.
    let _: () = conductors[1]
        .call(&bob, "set_signal", Some(Stance::Oppose))
        .await;
    let mine: ProjectView = conductors[1].call(&bob, "get_project", ()).await;
    assert_eq!(mine.my_signal, Some(Stance::Oppose));
    assert_eq!((mine.demand, mine.opposition), (0, 1));

    let _: () = conductors[1].call(&bob, "leave", ()).await;
    let _: ActionHash = conductors[0]
        .call(&ann, "update_charter", charter("Garden 2"))
        .await;
    await_consistency(&cells).await.expect("consistency");
    let after: ProjectView = conductors[0].call(&ann, "get_project", ()).await;
    assert_eq!(after.members, vec![cells[0].agent_pubkey().clone()]);
    assert_eq!(after.charter.map(|c| c.title), Some("Garden 2".into()));
    assert_eq!((after.demand, after.opposition), (0, 1));
}

#[tokio::test(flavor = "multi_thread")]
async fn new_members_start_two_projects_a_week() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let cells = common::directory_network(&mut conductors, 1).await;
    let (founder, eve) = (cells[0].zome("identity"), cells[1].zome("identity"));
    let (founder_key, eve_key) = (
        cells[0].agent_pubkey().clone(),
        cells[1].agent_pubkey().clone(),
    );

    let forged = Box::pin(conductors[1].call_fallible::<_, ActionHash>(
        &eve,
        "create_listing",
        listing(&founder_key, "x"),
    ))
    .await;
    assert!(
        forged.is_err(),
        "a listing can't name someone else as creator"
    );

    for seed in ["a", "b"] {
        let _: ActionHash =
            Box::pin(conductors[1].call(&eve, "create_listing", listing(&eve_key, seed))).await;
    }
    let left: Allowance = conductors[1].call(&eve, "project_allowance", ()).await;
    assert_eq!(left.remaining, Some(0));
    let third = Box::pin(conductors[1].call_fallible::<_, ActionHash>(
        &eve,
        "create_listing",
        listing(&eve_key, "c"),
    ))
    .await;
    assert!(
        third.is_err(),
        "a New member's third project this week is refused"
    );

    for seed in ["d", "e", "f"] {
        let _: ActionHash =
            Box::pin(conductors[0].call(&founder, "create_listing", listing(&founder_key, seed)))
                .await;
    }
    let unlimited: Allowance = conductors[0].call(&founder, "project_allowance", ()).await;
    assert_eq!(unlimited.remaining, None);

    await_consistency(&cells).await.expect("consistency");
    let all: Vec<ListingView> = conductors[1].call(&eve, "list_projects", ()).await;
    assert_eq!(all.len(), 5);
    let eves = all.iter().filter(|l| l.creator == eve_key).count();
    assert_eq!(eves, 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn only_the_creator_updates_a_listing() {
    let mut conductors = SweetConductorBatch::standard(2).await;
    let cells = common::directory_network(&mut conductors, 1).await;
    let (founder, eve) = (cells[0].zome("identity"), cells[1].zome("identity"));
    let founder_key = cells[0].agent_pubkey().clone();

    let id: ActionHash =
        Box::pin(conductors[0].call(&founder, "create_listing", listing(&founder_key, "g"))).await;
    let mut renamed = listing(&founder_key, "g");
    renamed.title = "Garden co-op".into();
    let update = ListingUpdate {
        id: id.clone(),
        listing: renamed,
    };
    let _: ActionHash = Box::pin(conductors[0].call(&founder, "update_listing", &update)).await;
    await_consistency(&cells).await.expect("consistency");

    let seen: Option<ListingView> = conductors[1].call(&eve, "get_listing", id.clone()).await;
    let seen = seen.expect("listing visible to others");
    assert_eq!(seen.listing.title, "Garden co-op");
    assert_eq!(seen.creator, founder_key);

    let hijack =
        Box::pin(conductors[1].call_fallible::<_, ActionHash>(&eve, "update_listing", &update))
            .await;
    assert!(hijack.is_err(), "only the creator updates a listing");
}
