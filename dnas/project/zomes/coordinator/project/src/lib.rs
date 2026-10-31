//! Project DNA API (docs/plans/M2-projects.md, M3-governance.md): charter, membership,
//! signals, and the project view with passed decisions applied (ADR 0033).

mod charter;
mod organizers;
mod own;
mod people;
mod scope_items;
mod spaces;

use hdk::prelude::*;
use project_integrity::{
    audience, space_creator, Applied, AppliedUpdate, Charter, LinkTypes, OrganizerRole,
};
use sp_governance::phases::{next_phase, stage_label, visible_phases, Phase, Shape};
use sp_governance::returns::returnable;
use space_scope::{enter, In};

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
pub fn base_charter(call: In<()>) -> ExternResult<Option<Charter>> {
    enter(call)?;
    Ok(charter::base()?.map(|(c, _)| c))
}

#[hdk_extern]
pub fn join(call: In<()>) -> ExternResult<()> {
    enter(call)?;
    people::join(&applied()?.removed)?;
    space_scope::touch()
}

#[hdk_extern]
pub fn leave(call: In<()>) -> ExternResult<()> {
    enter(call)?;
    people::leave()?;
    space_scope::touch()
}

#[hdk_extern]
pub fn set_signal(call: In<Option<Stance>>) -> ExternResult<()> {
    let stance = enter(call)?;
    if audience()?.organized() && stance.is_some() {
        return Err(wasm_error!(WasmErrorInner::Guest(
            "Organizer-controlled events don't use proposal signals.".into()
        )));
    }
    people::set_signal(stance)?;
    space_scope::touch()
}

#[hdk_extern]
pub fn add_organizer(call: In<AgentPubKey>) -> ExternResult<ActionHash> {
    let agent = enter(call)?;
    organizers::add(agent)
}

/// List one of your private events on this private community, for its members (ADR 0069).
#[hdk_extern]
pub fn add_scope_item(call: In<String>) -> ExternResult<()> {
    let seed = enter(call)?;
    scope_items::add(seed)
}

/// The private events listed on this community, newest first.
#[hdk_extern]
pub fn scope_items(call: In<()>) -> ExternResult<Vec<scope_items::ScopeItem>> {
    enter(call)?;
    scope_items::list()
}

/// What makes the caller an organizer, if anything (governance and activities cite it).
#[hdk_extern]
pub fn my_organizer_role(call: In<()>) -> ExternResult<Option<OrganizerRole>> {
    enter(call)?;
    organizers::role_of(&agent_info()?.agent_initial_pubkey)
}

#[hdk_extern]
pub fn my_membership(call: In<()>) -> ExternResult<Option<ActionHash>> {
    enter(call)?;
    people::my_membership()
}

/// The space a call is about: in a private clone, its one space (other zomes ask this).
#[hdk_extern]
pub fn space(call: In<()>) -> ExternResult<ActionHash> {
    enter(call)?;
    space_scope::current()
}

/// Mark the space active this hour, for a write in another zome (chat).
#[hdk_extern]
pub fn touch(call: In<()>) -> ExternResult<()> {
    enter(call)?;
    space_scope::touch()
}

#[hdk_extern]
pub fn member_snapshot(call: In<()>) -> ExternResult<Vec<ActionHash>> {
    enter(call)?;
    people::snapshot()
}

#[hdk_extern]
pub fn signal_counts(call: In<()>) -> ExternResult<SignalCounts> {
    enter(call)?;
    let p = people::read()?;
    Ok(SignalCounts {
        demand: p.demand,
        opposition: p.opposition,
    })
}

#[hdk_extern]
pub fn get_project(call: In<()>) -> ExternResult<Option<ProjectView>> {
    enter(call)?;
    let Some(creator) = space_creator(&space_scope::current()?)? else {
        return Ok(None);
    };
    let base = charter::base()?;
    let applied = applied()?;
    let people = people::read_without(&applied.removed)?;
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
    let (phases, next, returns, stage) = lifecycle(mode, subtype, current);

    let organizers = organizers::list()?;
    let seen = audience()?;
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
        returns,
        updates: applied.updates,
        members: people.members,
        demand: people.demand,
        opposition: people.opposition,
        am_member: people.am_member,
        my_signal: people.my_signal,
        created_at,
        last_activity_at,
        audience: seen.audience_id().into(),
        governance: seen.governance_id().into(),
        am_organizer: organizers.contains(&agent_info()?.agent_initial_pubkey),
        organizers,
    }))
}

/// The stepper's phases, the next phase, the returns and the stage label for this mode. A help
/// request, thread or post never moves (ADR 0067, 0070); `sp_governance` doesn't know its mode.
fn lifecycle(
    mode: &str,
    subtype: &str,
    current: Phase,
) -> (Vec<PhaseStep>, Option<PhaseStep>, Vec<PhaseStep>, String) {
    if mode == sp_help::MODE {
        return (Vec::new(), None, Vec::new(), "Help request".into());
    }
    // Threads and posts never move either (ADR 0070).
    if mode == sp_discussion::THREAD {
        return (Vec::new(), None, Vec::new(), "Thread".into());
    }
    if mode == sp_discussion::POST {
        return (Vec::new(), None, Vec::new(), "Post".into());
    }
    let shape = Shape {
        mode,
        subtype,
        include_access: false,
    };
    let step = |p: Phase| PhaseStep {
        id: p.id().to_string(),
        label: stage_label(shape, p).to_string(),
    };
    (
        visible_phases(shape, current)
            .into_iter()
            .map(step)
            .collect(),
        next_phase(shape, current).map(step),
        returnable(shape, current).into_iter().map(step).collect(),
        stage_label(shape, current).to_string(),
    )
}

/// Passed decisions, from the governance zome of this cell.
fn applied() -> ExternResult<Applied> {
    match call(
        CallTargetCell::Local,
        ZomeName::from("governance"),
        "applied".into(),
        None,
        space_scope::pass(())?,
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
    // The production or operations plan chooses the subtype (ADR 0039).
    if let Some(s) = &a.subtype {
        c.subtype.clone_from(s);
    }
    c
}

/// The current space's links of `link_type` (members, signals, organizers hang off it).
pub(crate) fn links(link_type: LinkTypes) -> ExternResult<Vec<Link>> {
    get_links(
        LinkQuery::try_new(space_scope::base()?, link_type)?,
        GetStrategy::default(),
    )
}
