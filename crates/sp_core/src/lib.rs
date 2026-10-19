//! Shared, dependency-free building blocks. Keep this crate tiny: anything with
//! behaviour belongs in a domain crate (`sp_governance`, `sp_index`).

pub mod chat;
pub mod handle;
pub mod limits;
pub mod network;
pub mod project;
