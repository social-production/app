//! jemalloc's own figures for `/readyz` (ADR 0064): `allocated` is what the program holds,
//! `resident` what jemalloc keeps in RAM, `retained` what it keeps mapped but returned. A big
//! gap between allocated and the process's RSS points at allocator overhead, not live data.

#[cfg(feature = "jemalloc")]
pub fn heap() -> serde_json::Value {
    use tikv_jemalloc_ctl::{epoch, stats};
    let mb = |bytes: Result<usize, tikv_jemalloc_ctl::Error>| bytes.ok().map(|b| b / (1024 * 1024));
    // The statistics are cached until the epoch advances.
    let _ = epoch::advance();
    serde_json::json!({
        "allocatedMb": mb(stats::allocated::read()),
        "activeMb": mb(stats::active::read()),
        "residentMb": mb(stats::resident::read()),
        "retainedMb": mb(stats::retained::read()),
    })
}

/// Builds without jemalloc (debug, tests) have no allocator figures.
#[cfg(not(feature = "jemalloc"))]
pub fn heap() -> serde_json::Value {
    serde_json::Value::Null
}
