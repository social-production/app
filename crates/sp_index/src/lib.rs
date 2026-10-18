//! Local, non-authoritative index over DHT data (ADR 0012). Each full node builds its
//! own from the records it holds; thin clients query the node they are connected to.
//! Ranking lives here so every node ranks the same way. Implemented in M2.
