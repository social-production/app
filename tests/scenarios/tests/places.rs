//! M6 places (ADR 0078): a charter, an edit and its passed location carry a place with
//! coordinates; validators refuse a place with more decimals than its precision, half a
//! coordinate pair, or a label other than the record's own.

use holochain::prelude::*;
use serde::{Deserialize, Serialize};

mod common;

use common::net::Net;

/// Mirrors `project_integrity::Place`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Place {
    label: String,
    latitude: Option<f64>,
    longitude: Option<f64>,
    region: Option<String>,
    country: Option<String>,
    provider_id: Option<String>,
    precision: String,
    online: bool,
}

/// Mirrors `project_integrity::Charter` with its place.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Charter {
    title: String,
    description: String,
    mode: String,
    subtype: String,
    location_label: String,
    place: Option<Place>,
    phase: String,
    tags: Vec<String>,
}

/// Mirrors `project_integrity::ProposalKind::Edit`.
#[derive(Serialize, Debug)]
enum ProposalKind {
    Edit {
        title: Option<String>,
        description: Option<String>,
        location: Option<String>,
        place: Option<Place>,
    },
}

#[derive(Serialize, Debug)]
struct VoteInput {
    proposal: ActionHash,
    choice: &'static str,
}

#[derive(Deserialize, Debug)]
struct ProjectView {
    charter: Option<Charter>,
}

fn place(label: &str, lat: f64, lon: f64) -> Place {
    Place {
        label: label.into(),
        latitude: Some(lat),
        longitude: Some(lon),
        region: Some("Victoria".into()),
        country: Some("Australia".into()),
        provider_id: None,
        precision: "approximate".into(),
        online: false,
    }
}

fn charter(title: &str, label: &str, place: Option<Place>) -> Charter {
    Charter {
        title: title.into(),
        description: "Growing food together.".into(),
        mode: "productive".into(),
        subtype: "standard".into(),
        location_label: label.into(),
        place,
        phase: "Proposal".into(),
        tags: Vec::new(),
    }
}

fn edit(label: &str, place: Option<Place>) -> ProposalKind {
    ProposalKind::Edit {
        title: None,
        description: None,
        location: Some(label.into()),
        place,
    }
}

async fn charter_of(net: &Net, i: usize) -> Charter {
    let seen: ProjectView = net.call(i, "project", "get_project", ()).await;
    seen.charter.expect("a charter")
}

#[tokio::test(flavor = "multi_thread")]
async fn charters_and_edits_carry_places() {
    let brunswick = place("Brunswick", -37.77, 144.96);
    let net = Net::with_charter(
        2,
        2,
        "places",
        charter("Garden", "Brunswick", Some(brunswick.clone())),
    )
    .await;
    assert_eq!(charter_of(&net, 0).await.place, Some(brunswick));

    let refused = [
        (
            "too precise",
            charter("A", "Fitzroy", Some(place("Fitzroy", -37.798, 144.98))),
        ),
        (
            "label differs",
            charter("B", "Carlton", Some(place("Fitzroy", -37.8, 144.98))),
        ),
        ("half a pair", {
            let mut half = place("Fitzroy", -37.8, 144.98);
            half.longitude = None;
            charter("C", "Fitzroy", Some(half))
        }),
    ];
    for (why, bad) in refused {
        let result: Result<ActionHash, _> = net.try_call(0, "project", "create_charter", bad).await;
        assert!(result.is_err(), "{why} is refused");
    }

    let wrong: Result<ActionHash, _> = net
        .try_call(
            0,
            "governance",
            "propose",
            edit("Carlton", Some(place("Fitzroy", -37.8, 144.98))),
        )
        .await;
    assert!(wrong.is_err(), "an edit's place carries its location label");

    let fitzroy = place("Fitzroy", -37.8, 144.98);
    let id: ActionHash = net
        .call(
            0,
            "governance",
            "propose",
            edit("Fitzroy", Some(fitzroy.clone())),
        )
        .await;
    net.settle().await;
    let vote = VoteInput {
        proposal: id,
        choice: "Yes",
    };
    let _: ActionHash = net.call(1, "governance", "vote", vote).await;
    net.settle().await;
    let moved = charter_of(&net, 1).await;
    assert_eq!(moved.location_label, "Fitzroy");
    assert_eq!(
        moved.place,
        Some(fitzroy),
        "the passed edit brings its place"
    );

    // A label-only edit leaves the place out (web: a typed label without a location).
    let id: ActionHash = net
        .call(0, "governance", "propose", edit("Somewhere north", None))
        .await;
    net.settle().await;
    let vote = VoteInput {
        proposal: id,
        choice: "Yes",
    };
    let _: ActionHash = net.call(1, "governance", "vote", vote).await;
    net.settle().await;
    let typed = charter_of(&net, 0).await;
    assert_eq!(typed.location_label, "Somewhere north");
    assert_eq!(typed.place, None);
    net.shut_down().await;
}
