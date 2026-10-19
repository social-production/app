//! Project values (web's Proposal phase "Add value", ADR 0041), ported from web-backend
//! `projects/actions.py` (add, importance votes), `projects/detail/hydrate.py` (averages) and
//! `plan_criteria.py` (values as plan criteria). Integer maths so every node agrees.

/// web's `AddValueSheet` input limit.
pub const LABEL_MAX: usize = 160;

/// Importance is 1 ("Unnecessary") to 10 ("Required").
#[must_use]
pub fn importance_ok(importance: u8) -> bool {
    (1..=10).contains(&importance)
}

/// Two labels that differ only in case or spacing are the same value (web's 409 "That value
/// is already listed", and `assessment_criteria_for_plan`'s `seen_labels`).
#[must_use]
pub fn label_key(label: &str) -> String {
    label
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Members' importance votes on one value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Importance {
    pub count: u32,
    pub sum: u32,
}

impl Importance {
    /// Invalid votes are skipped.
    #[must_use]
    pub fn of(votes: &[u8]) -> Self {
        let valid = votes.iter().copied().filter(|v| importance_ok(*v));
        valid.fold(Self::default(), |acc, v| Self {
            count: acc.count + 1,
            sum: acc.sum + u32::from(v),
        })
    }

    /// Average in hundredths, rounded half up (web rounds to 2 decimals); 0 when unrated.
    #[must_use]
    pub fn average_x100(&self) -> u32 {
        if self.count == 0 {
            return 0;
        }
        let (sum, count) = (u64::from(self.sum), u64::from(self.count));
        u32::try_from((sum * 200 + count) / (2 * count)).unwrap_or(u32::MAX)
    }

    #[must_use]
    pub fn prominent(&self) -> bool {
        is_prominent(self.average_x100())
    }
}

/// Values averaging 5 or more become plan criteria (web's `prominent_value_tuples`).
#[must_use]
pub fn is_prominent(average_x100: u32) -> bool {
    average_x100 >= 500
}

const CRITERION_PREFIX: &str = "value:";

/// A value's plan criterion id (web's `value_criterion_id`), from the value's id.
#[must_use]
pub fn criterion_id(value_id: &str) -> String {
    format!("{CRITERION_PREFIX}{value_id}")
}

/// The value id a criterion names, if it is a value criterion.
#[must_use]
pub fn parse_criterion(criterion: &str) -> Option<&str> {
    criterion
        .strip_prefix(CRITERION_PREFIX)
        .filter(|id| !id.is_empty())
}

/// web's value criterion question.
#[must_use]
pub fn criterion_label(label: &str) -> String {
    format!("How well does this plan satisfy \"{label}\"?")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn importance_is_one_to_ten() {
        assert!(!importance_ok(0));
        assert!(importance_ok(1) && importance_ok(10));
        assert!(!importance_ok(11));
    }

    #[test]
    fn averages_round_half_up_and_skip_invalid() {
        let i = Importance::of(&[7, 8, 0, 11]);
        assert_eq!((i.count, i.sum), (2, 15));
        assert_eq!(i.average_x100(), 750);
        assert_eq!(Importance::of(&[1, 1, 2]).average_x100(), 133);
        assert_eq!(Importance::of(&[]).average_x100(), 0);
    }

    #[test]
    fn prominent_from_five() {
        assert!(Importance::of(&[5]).prominent());
        assert!(Importance::of(&[4, 6]).prominent());
        assert!(!Importance::of(&[4, 5]).prominent());
        assert!(!Importance::of(&[]).prominent());
    }

    #[test]
    fn labels_match_ignoring_case_and_spacing() {
        assert_eq!(label_key("  Use  unused\tSpace "), "use unused space");
        assert_eq!(label_key("Use unused space"), label_key("use UNUSED space"));
    }

    #[test]
    fn criterion_ids_round_trip() {
        let id = criterion_id("uhCkk123");
        assert_eq!(id, "value:uhCkk123");
        assert_eq!(parse_criterion(&id), Some("uhCkk123"));
        assert_eq!(parse_criterion("value:"), None);
        assert_eq!(parse_criterion("rubric:achievability"), None);
        assert_eq!(
            criterion_label("Quiet"),
            "How well does this plan satisfy \"Quiet\"?"
        );
    }
}
