//! Project phases, ported from web-backend `services/projects/phases/labels.py`.
//! Phase ids keep web's numbering ("phase-1" … "phase-7"); 4 and 6 are internal steps that
//! display as their neighbours.

use serde::{Deserialize, Serialize};

/// The 66% demand share (of all signals) needed to leave the Proposal phase (web's signal gate).
pub const SIGNAL_GATE_BASIS_POINTS: u32 = 6_600;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Phase {
    P1,
    P2,
    P3,
    P4,
    P5,
    P6,
    P7,
}

impl Phase {
    pub const ALL: [Self; 7] = [
        Self::P1,
        Self::P2,
        Self::P3,
        Self::P4,
        Self::P5,
        Self::P6,
        Self::P7,
    ];

    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::P1 => "phase-1",
            Self::P2 => "phase-2",
            Self::P3 => "phase-3",
            Self::P4 => "phase-4",
            Self::P5 => "phase-5",
            Self::P6 => "phase-6",
            Self::P7 => "phase-7",
        }
    }

    #[must_use]
    pub fn parse(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.id() == id)
    }

    /// web's `effective_phase_id_for_progress`.
    fn effective(self) -> Self {
        match self {
            Self::P4 => Self::P3,
            Self::P6 => Self::P5,
            other => other,
        }
    }
}

/// The project shape that decides which phases exist (mode, subtype, access plan opted in).
#[derive(Clone, Copy, Debug)]
pub struct Shape<'a> {
    pub mode: &'a str,
    pub subtype: &'a str,
    pub include_access: bool,
}

impl Shape<'_> {
    fn skips_distribution(self) -> bool {
        self.subtype == "software"
    }
}

/// web's `display_stage_label`.
#[must_use]
pub fn stage_label(shape: Shape<'_>, phase: Phase) -> &'static str {
    let phase = phase.effective();
    match shape.mode {
        "personal-service" => {
            if phase == Phase::P2 {
                "Closed"
            } else {
                "Activity"
            }
        }
        "collective-service" => match phase {
            Phase::P2 => "Operations Plan",
            Phase::P3 => "Access Plan",
            Phase::P5 => "Activity",
            Phase::P7 => "Closed",
            _ => "Proposal",
        },
        _ => match phase {
            Phase::P2 => "Production Plan",
            Phase::P3 => "Distribution Plan",
            Phase::P5 => "Activity",
            Phase::P7 => "Closed",
            _ => "Proposal",
        },
    }
}

/// web's `visible_phase_ids_for_project`: the stepper's phases.
#[must_use]
pub fn visible_phases(shape: Shape<'_>, current: Phase) -> Vec<Phase> {
    use Phase::{P1, P2, P3, P5, P7};
    match shape.mode {
        "personal-service" if current == P2 => vec![P1, P2],
        "personal-service" => vec![P1],
        "collective-service" if shape.include_access || current == P3 => vec![P1, P2, P3, P5, P7],
        "collective-service" => vec![P1, P2, P5, P7],
        _ if shape.skips_distribution() => vec![P1, P2, P5, P7],
        _ => vec![P1, P2, P3, P5, P7],
    }
}

/// web's `next_phase_id_for_project`.
#[must_use]
pub fn next_phase(shape: Shape<'_>, current: Phase) -> Option<Phase> {
    if shape.mode == "personal-service" {
        return (current == Phase::P1).then_some(Phase::P2);
    }
    if current == Phase::P6 {
        return Some(Phase::P7);
    }
    Phase::ALL
        .into_iter()
        .filter(|p| *p > current)
        .find(|p| match p {
            Phase::P3 => {
                !(shape.skips_distribution()
                    || (shape.mode == "collective-service" && !shape.include_access))
            }
            Phase::P4 | Phase::P6 => false,
            _ => true,
        })
}

/// Leaving the Proposal phase needs demand ≥ 66% of all signals (and at least one signal).
#[must_use]
pub fn signal_gate_met(demand: u32, opposition: u32) -> bool {
    let total = u64::from(demand) + u64::from(opposition);
    total > 0 && u64::from(demand) * 10_000 >= u64::from(SIGNAL_GATE_BASIS_POINTS) * total
}

#[cfg(test)]
mod tests {
    use super::Phase::{P1, P2, P3, P4, P5, P6, P7};
    use super::*;

    const PRODUCTIVE: Shape<'static> = Shape {
        mode: "productive",
        subtype: "standard",
        include_access: false,
    };
    const SOFTWARE: Shape<'static> = Shape {
        mode: "productive",
        subtype: "software",
        include_access: false,
    };
    const COLLECTIVE: Shape<'static> = Shape {
        mode: "collective-service",
        subtype: "standard",
        include_access: false,
    };
    const PERSONAL: Shape<'static> = Shape {
        mode: "personal-service",
        subtype: "standard",
        include_access: false,
    };

    #[test]
    fn labels_match_web() {
        assert_eq!(stage_label(PRODUCTIVE, P2), "Production Plan");
        assert_eq!(stage_label(PRODUCTIVE, P4), "Distribution Plan");
        assert_eq!(stage_label(COLLECTIVE, P2), "Operations Plan");
        assert_eq!(stage_label(COLLECTIVE, P6), "Activity");
        assert_eq!(stage_label(PERSONAL, P1), "Activity");
        assert_eq!(stage_label(PERSONAL, P2), "Closed");
    }

    #[test]
    fn next_phase_skips_like_web() {
        assert_eq!(next_phase(PRODUCTIVE, P1), Some(P2));
        assert_eq!(next_phase(PRODUCTIVE, P2), Some(P3));
        assert_eq!(next_phase(PRODUCTIVE, P3), Some(P5));
        assert_eq!(next_phase(SOFTWARE, P2), Some(P5));
        assert_eq!(next_phase(COLLECTIVE, P2), Some(P5));
        let with_access = Shape {
            include_access: true,
            ..COLLECTIVE
        };
        assert_eq!(next_phase(with_access, P2), Some(P3));
        assert_eq!(next_phase(PRODUCTIVE, P6), Some(P7));
        assert_eq!(next_phase(PRODUCTIVE, P7), None);
        assert_eq!(next_phase(PERSONAL, P1), Some(P2));
        assert_eq!(next_phase(PERSONAL, P2), None);
    }

    #[test]
    fn visible_phases_match_web() {
        assert_eq!(visible_phases(PRODUCTIVE, P1), vec![P1, P2, P3, P5, P7]);
        assert_eq!(visible_phases(SOFTWARE, P1), vec![P1, P2, P5, P7]);
        assert_eq!(visible_phases(COLLECTIVE, P3), vec![P1, P2, P3, P5, P7]);
        assert_eq!(visible_phases(PERSONAL, P1), vec![P1]);
    }

    #[test]
    fn signal_gate_is_two_thirds_of_signals() {
        assert!(!signal_gate_met(0, 0));
        assert!(signal_gate_met(2, 1)); // 66.7%
        assert!(!signal_gate_met(1, 1));
        assert!(Phase::parse("phase-5") == Some(P5) && Phase::parse("x").is_none());
    }
}
