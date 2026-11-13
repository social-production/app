//! Project plans, ported from web-backend `services/projects_plans.py` (kinds and phases),
//! `services/plan_criteria.py` (rubric, ratings, leader) and `projects/phases/gates.py`
//! (the leading-plan gate). Integer maths so every node picks the same leader (ADR 0039).

use std::cmp::Ordering;

use serde::{Deserialize, Serialize};

use crate::phases::{Phase, Shape, EVENT, PROPOSAL};
pub use crate::rubric::{criteria, Criterion};

/// web's `phase_kind`. "organisation" is web's id for the operations plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PlanKind {
    Production,
    Distribution,
    Organisation,
    Access,
    /// An event plan, with its schedule (ADR 0057).
    Event,
}

impl PlanKind {
    pub const ALL: [Self; 5] = [
        Self::Production,
        Self::Distribution,
        Self::Organisation,
        Self::Access,
        Self::Event,
    ];

    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::Production => "production",
            Self::Distribution => "distribution",
            Self::Organisation => "organisation",
            Self::Access => "access",
            Self::Event => "event",
        }
    }

    #[must_use]
    pub fn parse(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.id() == id)
    }

    /// web's `PLAN_TYPE_REQUIRED_PHASE`: plans are submitted only in this phase.
    #[must_use]
    pub fn phase(self) -> Phase {
        match self {
            Self::Production | Self::Organisation | Self::Event => Phase::P2,
            Self::Distribution | Self::Access => Phase::P3,
        }
    }

    /// web's `ALLOWED_PLAN_TYPES_BY_MODE`.
    #[must_use]
    pub fn allowed(self, mode: &str) -> bool {
        match mode {
            "productive" => matches!(self, Self::Production | Self::Distribution),
            "collective-service" => matches!(self, Self::Organisation | Self::Access),
            EVENT => self == Self::Event,
            // A proposal takes every kind that can give it a type (ADR 0059).
            PROPOSAL => self.makes().is_some(),
            _ => false,
        }
    }

    /// The type a proposal becomes when this plan leads it out of the Plan phase (ADR 0059):
    /// a production plan makes a productive project, an operations plan a collective service,
    /// an event plan an event. Second-phase plans make nothing.
    #[must_use]
    pub fn makes(self) -> Option<&'static str> {
        match self {
            Self::Production => Some("productive"),
            Self::Organisation => Some("collective-service"),
            Self::Event => Some(EVENT),
            Self::Distribution | Self::Access => None,
        }
    }

    /// The kind of plan a project of `mode` takes in `phase`, if any.
    #[must_use]
    pub fn for_phase(mode: &str, phase: Phase) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|k| k.allowed(mode) && k.phase() == phase)
    }

    /// Every kind of plan a project of `mode` takes in `phase`: one for a typed project, the
    /// three type-giving kinds for a proposal in its Plan phase (ADR 0059).
    #[must_use]
    pub fn open_kinds(mode: &str, phase: Phase) -> Vec<Self> {
        Self::ALL
            .into_iter()
            .filter(|k| k.allowed(mode) && k.phase() == phase)
            .collect()
    }

    /// Production and operations plans choose the project's subtype (web's
    /// `sync_project_plan_leading_flags`).
    #[must_use]
    pub fn sets_subtype(self) -> bool {
        matches!(self, Self::Production | Self::Organisation)
    }
}

/// web's `_required_leading_plan_kind`: leaving `current` needs an approved leading plan of
/// this kind. Moving back or within Proposal/Activity needs none.
#[must_use]
pub fn required_plan_kind(shape: Shape<'_>, current: Phase) -> Option<PlanKind> {
    match current {
        Phase::P3 if shape.skips_distribution() => None,
        Phase::P2 | Phase::P3 => PlanKind::for_phase(shape.mode, current),
        _ => None,
    }
}

/// As `required_plan_kind`, for a proposal too: leaving `current` needs an approved leading
/// plan of any one of these kinds; empty when it needs none.
#[must_use]
pub fn required_plan_kinds(shape: Shape<'_>, current: Phase) -> Vec<PlanKind> {
    match current {
        Phase::P3 if shape.skips_distribution() => Vec::new(),
        Phase::P2 | Phase::P3 => PlanKind::open_kinds(shape.mode, current),
        _ => Vec::new(),
    }
}

/// A criterion rating is 1 (strongly oppose) to 5 (strongly support).
#[must_use]
pub fn rating_ok(rating: u8) -> bool {
    (1..=5).contains(&rating)
}

/// web's `criterion_rating_summary` for one criterion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ratings {
    pub count: u32,
    pub sum: u32,
    /// How many gave 1, 2, 3, 4, 5.
    pub distribution: [u32; 5],
}

impl Ratings {
    /// Average in hundredths, rounded half up (web rounds to 2 decimals); 0 when unrated.
    #[must_use]
    pub fn average_x100(&self) -> u32 {
        if self.count == 0 {
            return 0;
        }
        let (sum, count) = (u64::from(self.sum), u64::from(self.count));
        u32::try_from((sum * 200 + count) / (2 * count)).unwrap_or(u32::MAX)
    }
}

/// Invalid ratings are skipped, as web does.
#[must_use]
pub fn summarize(ratings: &[u8]) -> Ratings {
    let mut out = Ratings::default();
    for r in ratings.iter().copied().filter(|r| rating_ok(*r)) {
        out.count += 1;
        out.sum += u32::from(r);
        if let Some(slot) = out.distribution.get_mut(usize::from(r - 1)) {
            *slot += 1;
        }
    }
    out
}

/// A plan's average: the mean of its rated criteria's averages, kept as a fraction
/// (`sum of hundredths / criteria`) so comparisons are exact (web's `plan_average_rating`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Score {
    pub sum_x100: u64,
    pub rated: u64,
}

impl Score {
    #[must_use]
    pub fn of(criteria: &[Ratings]) -> Self {
        let rated: Vec<&Ratings> = criteria.iter().filter(|r| r.count > 0).collect();
        Self {
            sum_x100: rated.iter().map(|r| u64::from(r.average_x100())).sum(),
            rated: u64::try_from(rated.len()).unwrap_or(u64::MAX),
        }
    }

    /// Average in hundredths (rounded down), `None` when nothing is rated.
    #[must_use]
    pub fn average_x100(&self) -> Option<u32> {
        (self.rated > 0).then(|| u32::try_from(self.sum_x100 / self.rated).unwrap_or(u32::MAX))
    }

    fn cmp_value(&self, other: &Self) -> Ordering {
        let (a, b) = (
            fraction(self.sum_x100, self.rated),
            fraction(other.sum_x100, other.rated),
        );
        (u128::from(a.0) * u128::from(b.1)).cmp(&(u128::from(b.0) * u128::from(a.1)))
    }
}

/// An unrated plan counts as 0, like web's `0.0`.
fn fraction(num: u64, den: u64) -> (u64, u64) {
    if den == 0 {
        (0, 1)
    } else {
        (num, den)
    }
}

/// An approved plan competing to lead its kind.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Candidate<T> {
    pub id: T,
    pub yes: u32,
    pub no: u32,
    pub score: Score,
}

impl<T> Candidate<T> {
    fn cmp_ratio(&self, other: &Self) -> Ordering {
        let a = fraction(u64::from(self.yes), u64::from(self.yes + self.no));
        let b = fraction(u64::from(other.yes), u64::from(other.yes + other.no));
        (u128::from(a.0) * u128::from(b.1)).cmp(&(u128::from(b.0) * u128::from(a.1)))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Leader<T> {
    None,
    One(T),
    /// No unique leader: these share the best approval and the best average (web's "tied").
    Tied(Vec<T>),
}

/// web's `pick_leading_plan_id`: the best approval ratio, then the best average rating;
/// a tie on both means no leader.
#[must_use]
pub fn pick_leader<T: Clone>(candidates: &[Candidate<T>]) -> Leader<T> {
    let Some(best_ratio) = candidates.iter().max_by(|a, b| a.cmp_ratio(b)) else {
        return Leader::None;
    };
    let top: Vec<&Candidate<T>> = candidates
        .iter()
        .filter(|c| c.cmp_ratio(best_ratio) == Ordering::Equal)
        .collect();
    let Some(best_score) = top.iter().max_by(|a, b| a.score.cmp_value(&b.score)) else {
        return Leader::None;
    };
    let best: Vec<T> = top
        .iter()
        .filter(|c| c.score.cmp_value(&best_score.score) == Ordering::Equal)
        .map(|c| c.id.clone())
        .collect();
    match best.as_slice() {
        [one] => Leader::One(one.clone()),
        _ => Leader::Tied(best),
    }
}

#[cfg(test)]
#[path = "plans_tests.rs"]
mod tests;
