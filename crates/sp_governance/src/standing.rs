//! Standing levels (docs/GOVERNANCE.md#standing). In P2P, standing is never counted
//! live: a level is held only through a validated `StandingGrant` that cites its evidence.
//!
//! Change from the `FastAPI` rules (ADR 0008): only `Full` votes count. `Warm` is kept as a
//! visible progress step but carries no vote weight.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StandingLevel {
    Paused,
    New,
    Warm,
    Full,
}

impl StandingLevel {
    #[must_use]
    pub fn can_vote(self) -> bool {
        self == Self::Full
    }

    #[must_use]
    pub fn can_vouch(self) -> bool {
        self == Self::Full
    }
}

#[cfg(test)]
mod tests {
    use super::StandingLevel;

    #[test]
    fn only_full_votes() {
        assert!(StandingLevel::Full.can_vote());
        assert!(!StandingLevel::Warm.can_vote());
        assert!(!StandingLevel::Paused.can_vote());
    }
}
