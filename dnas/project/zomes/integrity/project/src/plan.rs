//! Plans (M3b, ADR 0039): a plan is a `Proposal` whose yes/no votes approve it through the
//! usual Outcome; members also rate it against web's rubric. Ratings only break ties between
//! approved plans, so they can change (the latest per criterion counts) and stay out of
//! Outcome validation. Criteria are web's rubric plus the project's values (ADR 0041).

use hdi::prelude::*;
use holo_hash::ActionHashB64;
use sp_core::plan::{check_plan, PlanFields, StageFields};
use sp_event::{check_schedule, ScheduleFields};
use sp_governance::plans::{criteria, rating_ok, PlanKind};
use sp_governance::values::parse_criterion;

use crate::links::is_membership;
use crate::value::is_value;
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
    /// Event plans only, and required there (ADR 0057).
    #[serde(default)]
    pub schedule: Option<EventSchedule>,
}

/// web's `EventPlanScheduleInput`, in UTC wall time (ADR 0057).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EventSchedule {
    pub mode: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub start_time: Option<String>,
    pub finish_time: Option<String>,
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
    check_event_fields(plan)?;
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

/// An event plan names its place and schedule; other plans have no schedule.
fn check_event_fields(plan: &Plan) -> Result<(), String> {
    match (plan.kind, &plan.schedule) {
        (PlanKind::Event, Some(s)) => {
            if plan.location.as_deref().is_none_or(|l| l.trim().is_empty()) {
                return Err("an event plan needs a place".into());
            }
            check_schedule(&ScheduleFields {
                mode: &s.mode,
                start_date: s.start_date.as_deref(),
                end_date: s.end_date.as_deref(),
                start_time: s.start_time.as_deref(),
                finish_time: s.finish_time.as_deref(),
            })
        }
        (PlanKind::Event, None) => Err("an event plan needs a schedule".into()),
        (_, Some(_)) => Err("only event plans have a schedule".into()),
        (_, None) => Ok(()),
    }
}

/// The plan a record proposes, if it is a plan proposal.
pub fn plan_of(hash: &ActionHash) -> ExternResult<Option<Plan>> {
    Ok(plan_with_space(hash)?.map(|(plan, _)| plan))
}

/// The plan proposal at `hash` and the space it is for.
fn plan_with_space(hash: &ActionHash) -> ExternResult<Option<(Plan, ActionHash)>> {
    match crate::typed_entry(&must_get_valid_record(hash.clone())?)? {
        Some(EntryTypes::Proposal(p)) => Ok(match p.kind {
            ProposalKind::Plan(plan) => Some((plan, p.space)),
            _ => None,
        }),
        _ => Ok(None),
    }
}

pub fn validate_rating(
    r: &Rating,
    action: &TypedAction<CreateData>,
) -> ExternResult<ValidateCallbackResult> {
    let Some((plan, space)) = plan_with_space(&r.proposal)? else {
        return invalid("a rating must be on a plan");
    };
    if !criterion_ok(&plan, &r.criterion)? {
        return invalid("unknown plan criterion");
    }
    if r.rating.is_some_and(|v| !rating_ok(v)) {
        return invalid("a rating is 1 to 5");
    }
    if !is_membership(&r.membership, action.author(), Some(action.timestamp()), Some(&space))? {
        return invalid("only members can rate plans");
    }
    Ok(ValidateCallbackResult::Valid)
}

/// A rubric question of the plan's kind, or any of the project's values (web accepts a rating
/// on every value; only prominent ones are shown).
fn criterion_ok(plan: &Plan, criterion: &str) -> ExternResult<bool> {
    if let Some(id) = parse_criterion(criterion) {
        return match ActionHashB64::from_b64_str(id) {
            Ok(hash) => is_value(&hash.into()),
            Err(_) => Ok(false),
        };
    }
    Ok(criteria(plan.kind, plan.subtype_or_standard())
        .iter()
        .any(|c| c.id == criterion))
}
