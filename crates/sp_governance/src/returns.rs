//! Returning a project to an earlier phase, ported from web-backend
//! `services/projects/phases/revert.py` (the rule) and `projects/detail/hydrate.py`
//! (`revertablePhaseIds`, the choices the Return sheet offers). ADR 0040.

use crate::phases::{visible_phases, Phase, Shape, EVENT};

/// web's `revertablePhaseIds`: the phases up to the second plan phase (`order <= 3`) that come
/// before the current one and that this project shows. Personal services take no proposals
/// (web's `_ensure_phase_requests_allowed`); they reopen directly, which arrives with M5.
#[must_use]
pub fn returnable(shape: Shape<'_>, current: Phase) -> Vec<Phase> {
    if shape.mode == "personal-service" {
        return Vec::new();
    }
    // web's event lifecycle offers every earlier phase (`order < current_order`).
    if shape.mode == EVENT {
        return visible_phases(shape, current)
            .into_iter()
            .filter(|p| *p < current)
            .collect();
    }
    visible_phases(shape, current)
        .into_iter()
        .filter(|p| *p < current && *p <= Phase::P3)
        .collect()
}

/// web refuses a target that is not earlier than the current phase; this narrows it to the
/// phases the Return sheet offers.
#[must_use]
pub fn can_return(shape: Shape<'_>, current: Phase, to: Phase) -> bool {
    returnable(shape, current).contains(&to)
}

/// Going back to Proposal or the production/operations plan phase reopens the subtype choice:
/// on web the leading plan of that phase sets the subtype whenever the leader changes, so the
/// subtype frozen by the earlier move stops applying (ADR 0040).
#[must_use]
pub fn reopens_subtype(to: Phase) -> bool {
    to <= Phase::P2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::phases::Phase::{P1, P2, P3, P5, P7};

    fn shape(mode: &'static str, subtype: &'static str, include_access: bool) -> Shape<'static> {
        Shape {
            mode,
            subtype,
            include_access,
        }
    }

    #[test]
    fn only_earlier_planning_phases_the_project_shows() {
        let productive = shape("productive", "standard", false);
        assert!(returnable(productive, P1).is_empty());
        assert_eq!(returnable(productive, P2), vec![P1]);
        assert_eq!(returnable(productive, P3), vec![P1, P2]);
        assert_eq!(returnable(productive, P5), vec![P1, P2, P3]);
        assert_eq!(returnable(productive, P7), vec![P1, P2, P3]);
    }

    #[test]
    fn events_return_to_any_earlier_phase() {
        let event = shape("event", "standard", false);
        assert_eq!(returnable(event, P5), vec![P1, P2]);
        assert_eq!(returnable(event, P7), vec![P1, P2, P5]);
    }

    #[test]
    fn hidden_phases_are_never_offered() {
        // Software skips distribution; collective services show Access Plan only when opted in.
        assert_eq!(
            returnable(shape("productive", "software", false), P5),
            vec![P1, P2]
        );
        assert_eq!(
            returnable(shape("collective-service", "standard", false), P5),
            vec![P1, P2]
        );
        assert_eq!(
            returnable(shape("collective-service", "standard", true), P5),
            vec![P1, P2, P3]
        );
        assert!(returnable(shape("personal-service", "standard", false), P2).is_empty());
    }

    #[test]
    fn a_return_goes_back_only() {
        let productive = shape("productive", "standard", false);
        assert!(can_return(productive, P5, P2));
        assert!(!can_return(productive, P2, P2), "not the current phase");
        assert!(!can_return(productive, P2, P3), "not forward");
        assert!(
            !can_return(productive, P7, P5),
            "Activity is not a planning phase"
        );
    }

    #[test]
    fn subtype_reopens_up_to_the_first_plan_phase() {
        assert!(reopens_subtype(P1) && reopens_subtype(P2));
        assert!(!reopens_subtype(P3));
    }
}
