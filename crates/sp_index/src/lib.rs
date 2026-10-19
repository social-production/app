//! Local, non-authoritative index over DHT data (ADR 0012). Each full node builds its
//! own from the records it holds; thin clients query the node they are connected to.
//! Ranking lives here so every node ranks the same way.

mod feed;
mod ranking;
mod search;
mod window;

pub use feed::{query, FeedQuery};
pub use ranking::{top_score, trending_score, ItemKind, ItemStats, Sort};
pub use search::matches;
pub use window::{window_bounds, Window};

/// Microseconds since the Unix epoch (Holochain's `Timestamp` unit).
pub type Micros = i64;

pub const MICROS_PER_HOUR: Micros = 3_600_000_000;
pub const MICROS_PER_DAY: Micros = 24 * MICROS_PER_HOUR;
