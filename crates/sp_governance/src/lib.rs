//! Governance rules shared by integrity zomes (validation), coordinator zomes (UI hints)
//! and the gateway. Everything here is a pure function of its inputs so that every node
//! reaches the same verdict. See docs/GOVERNANCE.md.

pub mod phases;
pub mod plans;
pub mod projects;
pub mod quorum;
pub mod standing;
pub mod tally;

pub use quorum::required_votes;
pub use standing::StandingLevel;
pub use tally::{tally, Choice, Tally, Threshold, Verdict};
