//! Seed-node gateway (docs/IDENTITY.md): an embedded Holochain conductor plus the HTTP API
//! that lets browsers sign up, sign in and talk to their hosted agent. `main.rs` runs it.

mod auth;
mod config;
mod hex;
mod node;
mod routes;
mod store;

use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;

pub use config::{keystore_passphrase, Config};

/// Boot the node and serve until `shutdown` resolves. `on_bound` gets the bound address.
pub async fn serve(
    config: Config,
    passphrase: Vec<u8>,
    on_bound: impl FnOnce(SocketAddr),
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let node = Box::pin(node::Node::start(&config, passphrase)).await?;
    tokio::spawn(background(node.clone(), config.happ_path.clone()));
    let state = routes::AppState {
        store: store::Store::open(&config.data_dir).await?,
        passkeys: Arc::new(auth::passkeys::Passkeys::new(
            &config.rp_id,
            &config.rp_origin,
        )?),
        pow: Arc::new(auth::pow::Pow::new(config.pow_bits)),
        signup_limit: Arc::new(auth::ratelimit::SignupLimit::new(
            config.signups_per_ip_per_day,
        )),
        // Phrase sign-ins: generous, just enough to stop hammering.
        signin_limit: Arc::new(auth::ratelimit::SignupLimit::new(200)),
        config: Arc::new(config),
        node,
    };
    let listener = tokio::net::TcpListener::bind(state.config.bind_addr).await?;
    let bound = listener.local_addr()?;
    tracing::info!(addr = %bound, "gateway listening");
    on_bound(bound);
    axum::serve(listener, routes::router(state))
        .with_graceful_shutdown(shutdown)
        .await?;
    Ok(())
}

/// After start: compile the zomes, swap in new coordinators, then keep the archivist
/// joining new projects and refreshing the feed index.
async fn background(node: node::Node, happ: std::path::PathBuf) {
    let started = std::time::Instant::now();
    match Box::pin(node.warm_up(&happ)).await {
        Ok(()) => tracing::info!(
            secs = started.elapsed().as_secs(),
            "zomes compiled (warm-up)"
        ),
        Err(e) => tracing::warn!(error = %e, "warm-up failed; first signup will be slower"),
    }
    match node.update_coordinators().await {
        Ok(dnas) => tracing::info!(dnas, "coordinators up to date"),
        Err(e) => tracing::warn!(error = %e, "coordinator update failed"),
    }
    let mut last = usize::MAX;
    loop {
        match Box::pin(node.archive_round(&happ)).await {
            Ok(count) if count != last => {
                tracing::info!(projects = count, "project index refreshed");
                last = count;
            }
            Ok(_) => {}
            Err(e) => tracing::warn!(error = %e, "archivist round failed"),
        }
        tokio::time::sleep(ARCHIVE_EVERY).await;
    }
}

const ARCHIVE_EVERY: std::time::Duration = std::time::Duration::from_secs(15);
