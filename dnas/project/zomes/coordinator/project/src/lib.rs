//! Project DNA API (docs/plans/M2-projects.md, M3-governance.md): charter, membership,
//! signals, and the project view with passed decisions applied (ADR 0033).

mod charter;
mod own;
mod people;

use hdk::prelude::*;
use project_integrity::{creator, Applied, AppliedUpdate, Charter, LinkTypes};
use sp_governance::phases::{next_phase, stage_label, visible_phases, Phase, Shape};

pub(crate) const CHARTER: &str = "charter";
pub(crate) const MEMBERS: &str = "members";
pub(crate) const SIGNALS: &str = "signals";

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stance {
    Demand,
    Oppose,
}

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
    pub updates: Vec<AppliedUpdate>,
    pub members: Vec<AgentPubKey>,
    pub demand: u32,
    pub opposition: u32,
    pub am_member: bool,
    pub my_signal: Option<Stance>,
    pub created_at: Option<Timestamp>,
    pub last_activity_at: Option<Timestamp>,
}

/// One phase as the stepper shows it (ids and labels from `sp_governance::phases`).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PhaseStep {
    pub id: String,
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignalCounts {
    pub demand: u32,
    pub opposition: u32,
}

#[hdk_extern]
pub fn create_charter(charter: Charter) -> ExternResult<ActionHash> {
    charter::create(charter)
}

#[hdk_extern]
pub fn base_charter() -> ExternResult<Option<Charter>> {
    Ok(charter::base()?.map(|(c, _)| c))
}

#[hdk_extern]
pub fn join() -> ExternResult<()> {
    people::join()
}

#[hdk_extern]
pub fn leave() -> ExternResult<()> {
    people::leave()
}

#[hdk_extern]
pub fn set_signal(stance: Option<Stance>) -> ExternResult<()> {
    people::set_signal(stance)
}

#[hdk_extern]
pub fn my_membership() -> ExternResult<Option<ActionHash>> {
    people::my_membership()
}

#[hdk_extern]
pub fn member_snapshot() -> ExternResult<Vec<ActionHash>> {
    people::snapshot()
}

#[hdk_extern]
pub fn signal_counts() -> ExternResult<SignalCounts> {
    let p = people::read()?;
    Ok(SignalCounts {
        demand: p.demand,
        opposition: p.opposition,
    })
}

#[hdk_extern]
pub fn get_project() -> ExternResult<Option<ProjectView>> {
    let Some(creator) = creator()? else {
        return Ok(None);
    };
    let base = charter::base()?;
    let applied = applied()?;
    let people = people::read()?;
    let created_at = base.as_ref().map(|(_, at)| *at);
    let charter = base.map(|(c, _)| merge(c, &applied));

    let current = applied
        .phase
        .as_deref()
        .and_then(Phase::parse)
        .unwrap_or(Phase::P1);
    let (mode, subtype) = charter.as_ref().map_or(("productive", "standard"), |c| {
        (c.mode.as_str(), c.subtype.as_str())
    });
    let shape = Shape {
        mode,
        subtype,
        include_access: false,
    };
    let step = |p: Phase| PhaseStep {
        id: p.id().to_string(),
        label: stage_label(shape, p).to_string(),
    };
    let phases = visible_phases(shape, current)
        .into_iter()
        .map(step)
        .collect();
    let next = next_phase(shape, current).map(step);
    let stage = stage_label(shape, current).to_string();

    let last_activity_at = [created_at, people.last_at, applied.last_decision_at]
        .into_iter()
        .flatten()
        .max();
    Ok(Some(ProjectView {
        creator,
        charter,
        phase: current.id().to_string(),
        stage,
        phases,
        next_phase: next,
        updates: applied.updates,
        members: people.members,
        demand: people.demand,
        opposition: people.opposition,
        am_member: people.am_member,
        my_signal: people.my_signal,
        created_at,
        last_activity_at,
    }))
}

/// Passed decisions, from the governance zome of this cell.
fn applied() -> ExternResult<Applied> {
    match call(
        CallTargetCell::Local,
        ZomeName::from("governance"),
        "applied".into(),
        None,
        (),
    )? {
        ZomeCallResponse::Ok(io) => io.decode().map_err(|e| wasm_error!(e)),
        other => Err(wasm_error!(WasmErrorInner::Guest(format!(
            "governance/applied: {other:?}"
        )))),
    }
}

fn merge(mut c: Charter, a: &Applied) -> Charter {
    if let Some(t) = &a.title {
        c.title.clone_from(t);
    }
    if let Some(d) = &a.description {
        c.description.clone_from(d);
    }
    if let Some(l) = &a.location {
        c.location_label.clone_from(l);
    }
    // The production or operations plan chooses the subtype (ADR 0038).
    if let Some(s) = &a.subtype {
        c.subtype.clone_from(s);
    }
    c
}

pub(crate) fn anchor(name: &str) -> ExternResult<EntryHash> {
    Path::from(name).path_entry_hash()
}

pub(crate) fn links(name: &str, link_type: LinkTypes) -> ExternResult<Vec<Link>> {
    get_links(
        LinkQuery::try_new(anchor(name)?, link_type)?,
        GetStrategy::default(),
    )
}
