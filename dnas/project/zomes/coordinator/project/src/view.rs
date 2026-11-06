//! What `get_project` returns.

use hdk::prelude::*;
use project_integrity::{AppliedRequests, AppliedUpdate, Charter};
use sp_service::Settings;

use crate::Stance;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProjectView {
    pub creator: AgentPubKey,
    /// The charter with every passed edit applied (and the subtype a plan chose).
    pub charter: Option<Charter>,
    /// web's phase id ("phase-1"…) and its label for this project's mode ("Production Plan").
    pub phase: String,
    pub stage: String,
    /// The phases this project shows, and the one a proposal may move it to next.
    pub phases: Vec<PhaseStep>,
    pub next_phase: Option<PhaseStep>,
    /// Earlier phases a return may go back to (web's `revertablePhaseIds`, ADR 0040).
    pub returns: Vec<PhaseStep>,
    pub updates: Vec<AppliedUpdate>,
    pub members: Vec<AgentPubKey>,
    pub demand: u32,
    pub opposition: u32,
    pub am_member: bool,
    pub my_signal: Option<Stance>,
    pub created_at: Option<Timestamp>,
    pub last_activity_at: Option<Timestamp>,
    /// web's event audience and governance, from the seed (ADR 0061).
    pub audience: String,
    pub governance: String,
    /// The creator and co-organizers (private events; just the creator otherwise).
    pub organizers: Vec<AgentPubKey>,
    pub am_organizer: bool,
    /// A service's request settings (ADR 0076): the newest saved or voted ones, else web's
    /// defaults for its mode.
    pub requests: RequestSettingsView,
}

/// One phase as the stepper shows it (ids and labels from `sp_governance::phases`).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PhaseStep {
    pub id: String,
    pub label: String,
}

/// web's `requestSystem.settings`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestSettingsView {
    pub enabled: bool,
    pub mode: String,
    pub off_schedule: bool,
}

impl RequestSettingsView {
    #[must_use]
    pub fn of(project_mode: &str, applied: Option<&AppliedRequests>) -> Self {
        let s = applied.map_or_else(
            || Settings::default_for(project_mode),
            |a| Settings {
                enabled: a.enabled,
                mode: a.mode.clone(),
                off_schedule: a.off_schedule,
            },
        );
        Self {
            enabled: s.enabled,
            mode: s.mode,
            off_schedule: s.off_schedule,
        }
    }
}
