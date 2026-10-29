//! Seed-node gateway binary. Everything lives in the library (`lib.rs`).

use tracing_subscriber::EnvFilter;

/// jemalloc instead of glibc malloc (ADR 0060): glibc's per-thread arenas kept freed memory,
/// so an idle seed node sat at ~1.1 GB; with jemalloc it is ~0.7 GB. It also replaces `malloc`
/// for C code such as `SQLite` (ADR 0066).
#[cfg(feature = "jemalloc")]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() -> anyhow::Result<()> {
    // Before any thread starts (ADR 0066): with the kernel's transparent huge pages set to
    // "always", as on Railway, each thread stack's first touch took a whole 2 MB page, so
    // every one of the conductor's 50–300 threads cost ~2 MB. The flag covers the process.
    // `KEEP_THP=1` leaves huge pages on (to measure the difference).
    #[cfg(target_os = "linux")]
    if std::env::var("KEEP_THP").as_deref() != Ok("1") {
        if let Err(e) = nix::sys::prctl::set_thp_disable(true) {
            eprintln!("could not turn off transparent huge pages: {e}");
        }
    }
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?
        .block_on(run())
}

async fn run() -> anyhow::Result<()> {
    // A hosted person's publish retries warn on every attempt when nobody else holds that part
    // of a network (NoPeersForLocation): on Railway that flooded past 500 lines/s and hid the
    // lines that matter. Kept quiet whatever RUST_LOG says; real failures still log as errors.
    let filter = EnvFilter::from_default_env()
        .add_directive("holochain::core::workflow::publish_dht_ops_workflow=error".parse()?);
    tracing_subscriber::fmt().with_env_filter(filter).init();
    let passphrase = gateway::keystore_passphrase()?;
    Box::pin(gateway::serve(
        gateway::Config::from_env(),
        passphrase,
        |_| {},
        shutdown(),
    ))
    .await
}

/// Railway sends SIGTERM on redeploy; ctrl-c covers local runs.
async fn shutdown() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let Ok(mut term) = signal(SignalKind::terminate()) else {
            return;
        };
        tokio::select! {
            _ = term.recv() => {}
            _ = tokio::signal::ctrl_c() => {}
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}
