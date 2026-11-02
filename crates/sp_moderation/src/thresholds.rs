//! web-backend's `services/moderation/thresholds.py`: how many votes a report needs, and what
//! share of them must be yes, to hide (serious harm only) or delete. Shares are whole
//! percentages, so "yes / total >= share" is `100 * yes >= share * total` (no floats).

use sp_governance::quorum::required_votes;

use crate::Reason;

/// web's floors, so a decision never looks like a one-person dogpile.
const MIN_DELETE_QUORUM: u32 = 3;
const MIN_SERIOUS_HARM_HIDE_QUORUM: u32 = 3;
const MIN_SERIOUS_HARM_DELETE_QUORUM: u32 = 5;
/// web's `MIN_APPROVAL_SHARE` (0.66).
const MIN_SHARE: u32 = 66;

/// Stability boost from content age (percentage points above 66%).
#[must_use]
pub fn age_boost_percent(age_days: u32) -> u32 {
    match age_days {
        0 => 0,
        1..=6 => 10,
        7..=29 => 20,
        30..=179 => 30,
        _ => 35,
    }
}

/// Stability boost from real participation (percentage points above 66%).
#[must_use]
pub fn popularity_boost_percent(score: u32) -> u32 {
    match score {
        0..=1 => 0,
        2..=7 => 5,
        8..=19 => 10,
        20..=49 => 15,
        _ => 20,
    }
}

/// web's `_popularity_score`: each distinct voter counts 1, each distinct commenter 2.
#[must_use]
pub fn popularity_score(voters: u32, commenters: u32) -> u32 {
    voters.saturating_add(commenters.saturating_mul(2))
}

/// Yes share needed to delete. Never below 66%.
#[must_use]
pub fn delete_share_percent(reason: Reason, age_days: u32, score: u32) -> u32 {
    let boost = age_boost_percent(age_days) + popularity_boost_percent(score);
    match reason {
        Reason::SeriousHarm => (MIN_SHARE + boost / 2).min(85),
        Reason::Spam => (MIN_SHARE + boost).min(90),
    }
}

/// Yes share needed to hide (serious harm only; spam never hides, web's 1.0).
#[must_use]
pub fn hide_share_percent(reason: Reason, age_days: u32, score: u32) -> u32 {
    match reason {
        Reason::Spam => 100,
        Reason::SeriousHarm => {
            let boost = age_boost_percent(age_days) + popularity_boost_percent(score);
            (MIN_SHARE + boost / 4).min(80)
        }
    }
}

/// Votes cast before deletion can pass. web's 1-vote DM path waits for message reports.
#[must_use]
pub fn delete_quorum(audience: u32, reason: Reason) -> u32 {
    let base = required_votes(audience);
    match reason {
        Reason::SeriousHarm if base == 0 => MIN_SERIOUS_HARM_DELETE_QUORUM,
        Reason::SeriousHarm => MIN_SERIOUS_HARM_DELETE_QUORUM.max(ceil_percent(base, 66)),
        Reason::Spam => MIN_DELETE_QUORUM.max(base),
    }
}

/// Votes cast before a serious-harm report hides the item.
#[must_use]
pub fn hide_quorum(audience: u32) -> u32 {
    let base = required_votes(audience);
    if base == 0 {
        MIN_SERIOUS_HARM_HIDE_QUORUM
    } else {
        MIN_SERIOUS_HARM_HIDE_QUORUM.max(ceil_percent(base, 33))
    }
}

/// `ceil(value * percent / 100)`, as web's `ceil(base * 0.66)`.
fn ceil_percent(value: u32, percent: u32) -> u32 {
    (value * percent).div_ceil(100)
}

/// One moment's bars: what the report menu shows and what the replay checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Thresholds {
    pub audience: u32,
    pub delete_quorum: u32,
    pub hide_quorum: u32,
    pub delete_share: u32,
    pub hide_share: u32,
}

impl Thresholds {
    #[must_use]
    pub fn at(reason: Reason, audience: u32, age_days: u32, score: u32) -> Self {
        Self {
            audience,
            delete_quorum: delete_quorum(audience, reason),
            hide_quorum: hide_quorum(audience),
            delete_share: delete_share_percent(reason, age_days, score),
            hide_share: hide_share_percent(reason, age_days, score),
        }
    }

    #[must_use]
    pub fn delete_ready(&self, yes: u32, no: u32) -> bool {
        ready(yes, no, self.delete_quorum, self.delete_share)
    }

    #[must_use]
    pub fn hide_ready(&self, reason: Reason, yes: u32, no: u32) -> bool {
        reason == Reason::SeriousHarm && ready(yes, no, self.hide_quorum, self.hide_share)
    }

    /// web's `can_still_reach_deletion`: if everyone left voted yes, could it still pass?
    #[must_use]
    pub fn can_still_delete(&self, yes: u32, no: u32) -> bool {
        let total = yes + no;
        let remaining = self.audience.saturating_sub(total);
        let max_total = total + remaining;
        max_total > 0
            && max_total >= self.delete_quorum
            && 100 * u64::from(yes + remaining)
                >= u64::from(self.delete_share) * u64::from(max_total)
    }
}

fn ready(yes: u32, no: u32, quorum: u32, share: u32) -> bool {
    let total = yes + no;
    total >= quorum && total > 0 && 100 * u64::from(yes) >= u64::from(share) * u64::from(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shares_follow_web() {
        // New, unpopular spam: 66%. A week old with 8 points: 66 + 20 + 10.
        assert_eq!(delete_share_percent(Reason::Spam, 0, 0), 66);
        assert_eq!(delete_share_percent(Reason::Spam, 7, 8), 90);
        assert_eq!(delete_share_percent(Reason::SeriousHarm, 7, 8), 81);
        assert_eq!(delete_share_percent(Reason::SeriousHarm, 400, 100), 85);
        assert_eq!(hide_share_percent(Reason::SeriousHarm, 7, 8), 73);
        assert_eq!(hide_share_percent(Reason::Spam, 0, 0), 100);
    }

    #[test]
    fn quorums_have_floors() {
        assert_eq!(delete_quorum(1, Reason::Spam), 3);
        assert_eq!(delete_quorum(3, Reason::Spam), 3);
        assert_eq!(delete_quorum(3, Reason::SeriousHarm), 5);
        assert_eq!(hide_quorum(3), 3);
        // web: required_votes(20) = 15; serious harm ceil(15 * 0.66) = 10; hide ceil(4.95) = 5.
        assert_eq!(delete_quorum(20, Reason::Spam), 15);
        assert_eq!(delete_quorum(20, Reason::SeriousHarm), 10);
        assert_eq!(hide_quorum(20), 5);
    }

    #[test]
    fn deletion_needs_quorum_and_share() {
        let t = Thresholds::at(Reason::Spam, 3, 0, 0);
        assert!(!t.delete_ready(2, 0));
        assert!(t.delete_ready(3, 0));
        assert!(t.delete_ready(2, 1)); // 66.7% >= 66%
        assert!(!t.delete_ready(1, 2));
        assert!(t.can_still_delete(1, 0));
        assert!(!t.can_still_delete(1, 2));
    }

    #[test]
    fn only_serious_harm_hides() {
        let t = Thresholds::at(Reason::SeriousHarm, 3, 0, 0);
        assert!(t.hide_ready(Reason::SeriousHarm, 3, 0));
        let s = Thresholds::at(Reason::Spam, 3, 0, 0);
        assert!(!s.hide_ready(Reason::Spam, 3, 0));
    }
}
