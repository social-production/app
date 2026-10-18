//! Gateway (docs/IDENTITY.md): the HTTP face of a seed/community node, run by `main.rs`.
//! Kept as a library so tests and future node tooling can start it in-process.
//! M0: health, `/session` for a shared demo agent, `/ws` proxy.
//! M1 adds passkey login and one hosted agent per user.

mod conductor;
mod config;
mod routes;
mod session;
mod ws_proxy;

use std::future::Future;
use std::net::SocketAddr;

pub use config::Config;

/// Provision the local conductor, bind `config.bind_addr`, and serve until `shutdown`
/// resolves. `on_bound` receives the actual address (useful with port 0).
pub async fn serve(
    config: Config,
    on_bound: impl FnOnce(SocketAddr),
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let conductor = conductor::Conductor::start(&config).await?;
    let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
    let bound = listener.local_addr()?;
    tracing::info!(addr = %bound, "gateway listening");
    on_bound(bound);

    let app = routes::router(routes::AppState { config, conductor });
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;
    Ok(())
}
