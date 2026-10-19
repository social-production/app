//! Vote tally for one proposal. Integer-only so validators agree exactly.
//! Ported from the per-type tallies in web-backend (e.g. `services/detail_links.py`).

use serde::{Deserialize, Serialize};

use crate::quorum::required_votes;

/// A counted ballot. Abstentions are not ballots; they never reach the tally.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Choice {
    Yes,
    No,
}

/// Approval threshold in basis points (6600 = 66%).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Threshold(pub u16);

impl Threshold {
    /// The platform default used by every decision kind today.
    pub const DEFAULT: Self = Self(6_600);

    fn met(self, yes: u32, total: u32) -> bool {
        total > 0 && u64::from(yes) * 10_000 >= u64::from(self.0) * u64::from(total)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tally {
    pub yes: u32,
    pub no: u32,
    pub population: u32,
    pub required: u32,
    pub passes: bool,
    pub can_still_pass: bool,
}

/// Count `choices` cast by eligible voters against a population of `population`.
#[must_use]
pub fn tally(choices: &[Choice], population: u32, threshold: Threshold) -> Tally {
    let yes = count(choices, Choice::Yes);
    let no = count(choices, Choice::No);
    let total = yes + no;
    let required = required_votes(population);
    let passes = total >= required && threshold.met(yes, total);

    let remaining = population.saturating_sub(total);
    let max_total = total + remaining;
    let can_still_pass =
        !passes && max_total >= required && threshold.met(yes + remaining, max_total);

    Tally {
        yes,
        no,
        population,
        required,
        passes,
        can_still_pass,
    }
}

/// Where a proposal stands. An Outcome may only be written for `Passed` or `Rejected`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verdict {
    Passed,
    /// Can no longer pass, even if every remaining member voted Yes.
    Rejected,
    Open,
}

impl Tally {
    #[must_use]
    pub fn verdict(&self) -> Verdict {
        if self.passes {
            Verdict::Passed
        } else if self.can_still_pass {
            Verdict::Open
        } else {
            Verdict::Rejected
        }
    }
}

fn count(choices: &[Choice], wanted: Choice) -> u32 {
    let n = choices.iter().filter(|c| **c == wanted).count();
    u32::try_from(n).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::{
        tally,
        Choice::{No, Yes},
        Threshold,
    };

    #[test]
    fn verdicts() {
        use super::Verdict;
        assert_eq!(
            tally(&[Yes], 1, Threshold::DEFAULT).verdict(),
            Verdict::Passed
        );
        assert_eq!(
            tally(&[Yes], 3, Threshold::DEFAULT).verdict(),
            Verdict::Open
        );
        assert_eq!(
            tally(&[No, No], 3, Threshold::DEFAULT).verdict(),
            Verdict::Rejected
        );
    }

    #[test]
    fn passes_with_quorum_and_two_thirds() {
        // population 3 → 3 votes required
        let t = tally(&[Yes, Yes, No], 3, Threshold::DEFAULT);
        assert!(t.passes, "2/3 = 66.7% ≥ 66%");
    }

    #[test]
    fn fails_without_quorum() {
        let t = tally(&[Yes, Yes], 10, Threshold::DEFAULT);
        assert_eq!(t.required, 8);
        assert!(!t.passes);
        assert!(t.can_still_pass);
    }

    #[test]
    fn cannot_pass_once_no_votes_block_it() {
        let t = tally(&[No, No, No, No], 10, Threshold::DEFAULT);
        assert!(!t.passes);
        assert!(!t.can_still_pass, "best case 6/10 = 60% < 66%");
    }

    #[test]
    fn empty_population_never_passes() {
        let t = tally(&[], 0, Threshold::DEFAULT);
        assert!(!t.passes);
    }
}
