//! When a plan became its kind's leader (web-backend's `prj-plan-lead` / `evt-plan-lead`,
//! sent to the new leader's author when a plan vote changes the leader). web keeps no history
//! of leaders; here the approved plans are replayed in the order they closed, and every close
//! that hands the lead to a different plan is one change (ADR 0072).

use hdk::prelude::*;
use sp_governance::plans::{Leader, PlanKind};

use crate::apply::{self, Standing};
use crate::plans::{self, Approved};
use crate::roll::Roll;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct LeadChange {
    /// The plan that took the lead, and its author (whom web notifies).
    pub plan: ActionHash,
    pub author: AgentPubKey,
    pub kind: PlanKind,
    /// When the plan whose vote changed the lead closed, and who closed it.
    pub at: Timestamp,
    pub by: AgentPubKey,
}

pub fn lead_changes() -> ExternResult<Vec<LeadChange>> {
    let state = apply::state(&mut Standing::default(), &Roll::load()?)?;
    Ok(replay(&state.approved, state.born()))
}

/// Each close, in order, against the leader among its rivals (its kind; in a proposal every
/// type-giving kind, ADR 0059) just before it. Ratings count as they
/// are now (web reads them live too; the app keeps no past averages).
pub(crate) fn replay(approved: &[Approved], born: &str) -> Vec<LeadChange> {
    let mut out = Vec::new();
    for (i, closed) in approved.iter().enumerate() {
        let kinds = plans::rivals(born, closed.kind);
        let before = plans::leader(&approved[..i], &kinds);
        let Leader::One(now) = plans::leader(&approved[..=i], &kinds) else {
            continue;
        };
        if before == Leader::One(now.clone()) {
            continue;
        }
        if let Some(lead) = approved[..=i].iter().find(|a| a.hash == now) {
            out.push(LeadChange {
                plan: now,
                author: lead.author.clone(),
                kind: closed.kind,
                at: closed.at,
                by: closed.closed_by.clone(),
            });
        }
    }
    out
}
