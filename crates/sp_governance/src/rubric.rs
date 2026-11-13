//! The questions members rate a plan on, ported from web-backend `services/plan_criteria.py`.

use crate::plans::PlanKind;

/// One rubric question. Value criteria (web's "How well does this plan satisfy …") come from
/// project values (`values`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Criterion {
    pub id: &'static str,
    pub label: &'static str,
}

const SHARED: [Criterion; 2] = [
    Criterion {
        id: "rubric:description-clarity",
        label: "Does the plan clearly explain what will happen and why?",
    },
    Criterion {
        id: "rubric:achievability",
        label: "Does this plan seem realistically achievable?",
    },
];
/// web's `EVENT_RUBRIC`.
const EVENT_RUBRIC: [Criterion; 2] = [
    Criterion {
        id: "rubric:timing-suitable",
        label: "Is the timing and schedule realistic?",
    },
    Criterion {
        id: "rubric:location-appropriate",
        label: "Is the location appropriate and accessible?",
    },
];
const PRODUCTION: Criterion = Criterion {
    id: "rubric:production-approach",
    label: "Is the proposed production approach appropriate?",
};
const SOFTWARE: Criterion = Criterion {
    id: "rubric:repository-clear",
    label: "Is the repository/setup clear enough?",
};
const DISTRIBUTION: Criterion = Criterion {
    id: "rubric:access-approach",
    label: "Is the access and request approach appropriate?",
};

/// web's `assessment_criteria_for_plan` (rubric part). `subtype` is the plan's own.
#[must_use]
pub fn criteria(kind: PlanKind, subtype: &str) -> Vec<Criterion> {
    let mut out = SHARED.to_vec();
    match kind {
        PlanKind::Production | PlanKind::Organisation => {
            out.push(PRODUCTION);
            if subtype == "software" {
                out.push(SOFTWARE);
            }
        }
        // web's `elif plan_kind == "distribution"`: access plans get only the shared rubric.
        PlanKind::Distribution => out.push(DISTRIBUTION),
        PlanKind::Access => {}
        PlanKind::Event => out.extend(EVENT_RUBRIC),
    }
    out
}
