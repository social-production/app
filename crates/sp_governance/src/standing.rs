//! Standing (docs/GOVERNANCE.md#standing). Standing is never counted live: an agent is
//! Full because it is a founder, or because a validated `StandingGrant` cites evidence that
//! every validator re-checks with these functions. Timestamps are microseconds since epoch.

use std::collections::HashSet;
use std::hash::Hash;

use serde::{Deserialize, Serialize};

const DAY_US: i64 = 24 * 60 * 60 * 1_000_000;

/// Distinct full-standing vouchers needed for Full.
pub const VOUCHES_FOR_FULL: usize = 2;
/// Vouches one agent may give per rolling window.
pub const VOUCH_CAP: usize = 5;
pub const VOUCH_CAP_WINDOW_US: i64 = 30 * DAY_US;
/// Steady activity: chain age and distinct active days.
pub const STEADY_MIN_AGE_US: i64 = 30 * DAY_US;
pub const STEADY_ACTIVE_DAYS: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StandingLevel {
    Paused,
    New,
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

/// One vouch, as validators see it after fetching the cited record.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VouchFact<A> {
    pub voucher: A,
    pub vouchee: A,
    pub at: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StandingError {
    WrongVouchee,
    SelfVouch,
    VouchAfterGrant,
    NotEnoughVouchers { distinct: usize },
    VouchCapReached,
    ChainTooYoung,
    TooFewActiveDays { days: usize },
}

impl std::fmt::Display for StandingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WrongVouchee => write!(f, "a cited vouch is for someone else"),
            Self::SelfVouch => write!(f, "agents cannot vouch for themselves"),
            Self::VouchAfterGrant => write!(f, "a cited vouch is newer than the grant"),
            Self::NotEnoughVouchers { distinct } => write!(
                f,
                "full standing needs {VOUCHES_FOR_FULL} distinct vouchers, got {distinct}"
            ),
            Self::VouchCapReached => write!(f, "vouch limit of {VOUCH_CAP} per 30 days reached"),
            Self::ChainTooYoung => write!(f, "steady activity needs a chain at least 30 days old"),
            Self::TooFewActiveDays { days } => write!(
                f,
                "steady activity needs {STEADY_ACTIVE_DAYS} active days, got {days}"
            ),
        }
    }
}

/// Vouch evidence for `grantee`'s Full grant made at `grant_at`.
/// Voucher standing is checked separately by the caller (it needs the DHT).
pub fn check_vouch_evidence<A: Eq + Hash>(
    grantee: &A,
    vouches: &[VouchFact<A>],
    grant_at: i64,
) -> Result<(), StandingError> {
    let mut vouchers = HashSet::new();
    for vouch in vouches {
        if &vouch.vouchee != grantee {
            return Err(StandingError::WrongVouchee);
        }
        if &vouch.voucher == grantee {
            return Err(StandingError::SelfVouch);
        }
        if vouch.at >= grant_at {
            return Err(StandingError::VouchAfterGrant);
        }
        vouchers.insert(&vouch.voucher);
    }
    if vouchers.len() < VOUCHES_FOR_FULL {
        return Err(StandingError::NotEnoughVouchers {
            distinct: vouchers.len(),
        });
    }
    Ok(())
}

/// A new vouch at `now`, given the times of the voucher's earlier vouches.
pub fn check_vouch_cap(earlier_vouches: &[i64], now: i64) -> Result<(), StandingError> {
    let recent = earlier_vouches
        .iter()
        .filter(|t| **t > now - VOUCH_CAP_WINDOW_US && **t <= now)
        .count();
    if recent >= VOUCH_CAP {
        return Err(StandingError::VouchCapReached);
    }
    Ok(())
}

/// Steady activity from the agent's own chain: first action at `chain_start`, every
/// action time in `action_times`, grant made at `grant_at`.
pub fn check_steady_activity(
    chain_start: i64,
    action_times: &[i64],
    grant_at: i64,
) -> Result<(), StandingError> {
    if grant_at - chain_start < STEADY_MIN_AGE_US {
        return Err(StandingError::ChainTooYoung);
    }
    let days: HashSet<i64> = action_times
        .iter()
        .filter(|t| **t <= grant_at)
        .map(|t| t.div_euclid(DAY_US))
        .collect();
    if days.len() < STEADY_ACTIVE_DAYS {
        return Err(StandingError::TooFewActiveDays { days: days.len() });
    }
    Ok(())
}

#[cfg(test)]
#[path = "standing_tests.rs"]
mod tests;
