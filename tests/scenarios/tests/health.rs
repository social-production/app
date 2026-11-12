//! M0 platform check in miniature: two agents on two conductors share each DNA.

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetConductorBatch};

mod common;

#[tokio::test(flavor = "multi_thread")]
async fn heartbeat_gossips_between_two_agents() {
    for dna_name in ["directory", "project", "conversation"] {
        let dna = common::load_dna(dna_name).await;
        let mut conductors = SweetConductorBatch::standard(2).await;
        let apps = conductors
            .setup_app("sp", &[dna])
            .await
            .expect("install app");
        conductors.exchange_peer_info().await;
        let cells = apps.cells_flattened();
        let (alice, bob) = (cells[0].zome("health"), cells[1].zome("health"));

        let pong: String = conductors[0].call(&alice, "ping", ()).await;
        assert_eq!(pong, "pong");

        let posted: ActionHash = conductors[0]
            .call(&alice, "post_heartbeat", "hello from alice".to_string())
            .await;
        await_consistency(&cells).await.expect("consistency");

        let seen: Vec<ActionHash> = conductors[1].call(&bob, "list_heartbeats", ()).await;
        assert_eq!(
            seen,
            vec![posted],
            "bob sees alice's heartbeat in {dna_name}"
        );
        common::shut_down(conductors).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn rejects_blank_heartbeat() {
    let dna = common::load_dna("directory").await;
    let mut conductors = SweetConductorBatch::standard(1).await;
    let apps = conductors
        .setup_app("sp", &[dna])
        .await
        .expect("install app");
    let cell = &apps.cells_flattened()[0];

    let result: Result<ActionHash, _> = conductors[0]
        .call_fallible(&cell.zome("health"), "post_heartbeat", "   ".to_string())
        .await;
    assert!(result.is_err(), "blank notes fail validation");
    common::shut_down(conductors).await;
}

#[derive(serde::Deserialize, Debug)]
struct Announced {
    note: String,
    author: AgentPubKey,
}

/// "Help the network" (ADR 0094): a desktop's announced bootstrap server reaches the others.
#[tokio::test(flavor = "multi_thread")]
async fn announced_servers_reach_other_agents() {
    let dna = common::load_dna("directory").await;
    let mut conductors = SweetConductorBatch::standard(2).await;
    let apps = conductors
        .setup_app("sp", &[dna])
        .await
        .expect("install app");
    conductors.exchange_peer_info().await;
    let cells = apps.cells_flattened();
    let (alice, bob) = (cells[0].zome("health"), cells[1].zome("health"));

    let note = r#"{"bootstrap":"http://203.0.113.5:8787","relay":"http://203.0.113.5:8787"}"#;
    let _: ActionHash = conductors[0]
        .call(&alice, "announce_server", note.to_string())
        .await;
    let too_long: Result<ActionHash, _> = conductors[0]
        .call_fallible(&alice, "announce_server", "x".repeat(281))
        .await;
    assert!(too_long.is_err(), "long notes are refused");
    await_consistency(&cells).await.expect("consistency");

    let heard: Vec<Announced> = conductors[1].call(&bob, "list_servers", ()).await;
    assert_eq!(heard.len(), 1);
    assert_eq!(heard[0].note, note);
    assert_eq!(&heard[0].author, cells[0].agent_pubkey());
    let heartbeats: Vec<ActionHash> = conductors[1].call(&bob, "list_heartbeats", ()).await;
    assert!(heartbeats.is_empty(), "announcements are not heartbeats");
    common::shut_down(conductors).await;
}
