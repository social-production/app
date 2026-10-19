//! Plans (M3b, ADR 0038): a plan is a `Proposal` whose yes/no votes approve it through the
//! usual Outcome; members also rate it against web's rubric. Ratings only break ties between
//! approved plans, so they can change (the latest per criterion counts) and stay out of
//! Outcome validation.

use hdi::prelude::*;
use sp_core::plan::{check_plan, PlanFields, StageFields};
use sp_governance::plans::{criteria, rating_ok, PlanKind};

use crate::links::is_membership;
use crate::{invalid, EntryTypes, ProposalKind};

/// web's plan wizard fields (web-backend `submit_project_plan`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Plan {
    pub kind: PlanKind,
    pub title: String,
    pub description: String,
    /// "How does this plan respond to current support?"
    pub demand_note: String,
    /// Production and operations plans choose the project's subtype.
    pub subtype: Option<String>,
    pub repository_url: Option<String>,
    pub location: Option<String>,
    pub recipients: Option<String>,
    pub stages: Vec<PlanStage>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlanStage {
    pub title: String,
    pub details: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl Plan {
    /// The subtype the rubric is chosen by.
    #[must_use]
    pub fn subtype_or_standard(&self) -> &str {
        self.subtype.as_deref().unwrap_or("standard")
    }
}

/// One member's rating of one criterion; `None` clears it. The latest per criterion counts.
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Rating {
    pub proposal: ActionHash,
    pub membership: ActionHash,
    pub criterion: String,
    pub rating: Option<u8>,
}

pub fn check(plan: &Plan) -> Result<(), String> {
    check_plan(&PlanFields {
        title: &plan.title,
        description: &plan.description,
        demand_note: &plan.demand_note,
        subtype: plan.subtype.as_deref(),
        repository_url: plan.repository_url.as_deref(),
        location: plan.location.as_deref(),
        recipients: plan.recipients.as_deref(),
        stages: plan
            .stages
            .iter()
            .map(|s| StageFields {
                title: &s.title,
                details: &s.details,
                start_date: s.start_date.as_deref(),
                end_date: s.end_date.as_deref(),
            })
            .collect(),
    })
}

/// The plan a record proposes, if it is a plan proposal.
pub fn plan_of(hash: &ActionHash) -> ExternResult<Option<Plan>> {
    match crate::typed_entry(&must_get_valid_record(hash.clone())?)? {
        Some(EntryTypes::Proposal(p)) => Ok(match p.kind {
            ProposalKind::Plan(plan) => Some(plan),
            _ => None,
        }),
        _ => Ok(None),
    }
}

pub fn validate_rating(
    r: &Rating,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    let Some(plan) = plan_of(&r.proposal)? else {
        return invalid("a rating must be on a plan");
    };
    if !criteria(plan.kind, plan.subtype_or_standard())
        .iter()
        .any(|c| c.id == r.criterion)
    {
        return invalid("unknown plan criterion");
    }
    if r.rating.is_some_and(|v| !rating_ok(v)) {
        return invalid("a rating is 1 to 5");
    }
    if !is_membership(&r.membership, action.author(), Some(action.timestamp()))? {
        return invalid("only members can rate plans");
    }
    Ok(ValidateCallbackResult::Valid)
}
