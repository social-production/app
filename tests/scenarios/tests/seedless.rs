//! SEEDLESS phase A (docs/plans/SEEDLESS.md, ADR 0094): with no Railway bootstrap anywhere,
//! three conductors use only the bootstrap server and relay a "Help the network" desktop runs
//! (`native/src/helping.rs`'s config). They find each other through it, share data, and keep
//! sharing once it stops.

use std::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use holochain::prelude::*;
use holochain::sweettest::{SweetConductorBatch, SweetConductorConfig, SweetZome};
use kitsune2_bootstrap_srv::{BootstrapSrv, Config};

mod common;

/// The desktop's server: production settings, plain HTTP, two workers, any free port.
fn helper() -> (BootstrapSrv, String) {
    let mut config = Config::production();
    config.worker_thread_count = 2;
    config.listen_address_list = vec![SocketAddr::from((Ipv4Addr::LOCALHOST, 0))];
    config.quic_bind_addr = None;
    let server = std::thread::spawn(move || BootstrapSrv::new(config))
        .join()
        .expect("server thread")
        .expect("bootstrap server");
    let url = format!("http://{}", server.listen_addrs()[0]);
    (server, url)
}

/// Poll `who`'s heartbeats until `note` shows up.
async fn sees(conductors: &SweetConductorBatch, who: usize, zome: &SweetZome, posted: &ActionHash) {
    for _ in 0..240 {
        let seen: Vec<ActionHash> = conductors[who].call(zome, "list_heartbeats", ()).await;
        if seen.contains(posted) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    panic!("conductor {who} never saw the heartbeat");
}

#[tokio::test(flavor = "multi_thread")]
async fn nodes_meet_through_a_desktops_bootstrap_and_keep_talking_after_it_stops() {
    let (mut server, url) = helper();
    // Every node's only server is the desktop's: Railway's is not in the list at all.
    let config = SweetConductorConfig::standard().tune_network_config(|n| {
        n.bootstrap_url = url2::url2!("{}", url);
        n.relay_url = url2::url2!("{}", url);
    });
    let mut conductors = SweetConductorBatch::from_config_rendezvous(3, config).await;
    let dna = common::load_dna("directory").await;
    // No `exchange_peer_info`: the nodes must find each other through the bootstrap.
    let apps = conductors.setup_app("sp", &[dna]).await.expect("install app");
    let cells = apps.cells_flattened();
    let zomes: Vec<SweetZome> = cells.iter().map(|c| c.zome("health")).collect();

    let first: ActionHash = conductors[0]
        .call(&zomes[0], "post_heartbeat", "found through a desktop".to_string())
        .await;
    sees(&conductors, 1, &zomes[1], &first).await;
    sees(&conductors, 2, &zomes[2], &first).await;

    // The desktop stops helping: what the nodes already know keeps them connected.
    server.shutdown().expect("stop the bootstrap server");
    let second: ActionHash = conductors[1]
        .call(&zomes[1], "post_heartbeat", "still here without it".to_string())
        .await;
    sees(&conductors, 0, &zomes[0], &second).await;
    sees(&conductors, 2, &zomes[2], &second).await;
    common::shut_down(conductors).await;
}
