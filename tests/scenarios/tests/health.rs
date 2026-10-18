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
}
