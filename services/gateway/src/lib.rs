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
    let node = node::Node::start(&config, passphrase).await?;
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
