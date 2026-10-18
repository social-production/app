//! Kitsune2 tuning shared by every node type (seed node, desktop/phone app), so all nodes
//! gossip on the same schedule (docs/NETWORK.md#observed-timings).
//!
//! Kitsune2's defaults suit large networks: a failed first round with a peer waits 5 min
//! before retrying, and rounds time out after 15 s, which relayed connections can miss.
//! For a small network, faster retries and longer timeouts make first contact much quicker.

/// Merged into the conductor's `network.advanced` config.
pub const GOSSIP_TUNING_JSON: &str = r#"{
  "k2Gossip": {
    "initiateIntervalMs": 30000,
    "minInitiateIntervalMs": 60000,
    "roundTimeoutMs": 60000
  }
}"#;
