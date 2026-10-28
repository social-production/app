//! Project values (ADR 0041), ported from web-backend `add_project_value` and
//! `vote_project_value_importance`: members add them and rate their importance (1–10) in the
//! Proposal phase; values averaging 5 or more become plan criteria.

use std::collections::BTreeMap;

use hdk::prelude::*;
use holo_hash::ActionHashB64;
use project_integrity::{EntryTypes, Importance, LinkTypes, UnitEntryTypes, Value};
use sp_governance::phases::Phase;
use sp_governance::values::{
    criterion_id, criterion_label, importance_ok, is_prominent, label_key, Importance as Summary,
    LABEL_MAX,
};

use crate::apply::{self, Standing};
use crate::calls;
use crate::reads::{self, Found};
use crate::roll::Roll;
use crate::write::guest;

/// web's `phaseOne.values` item.
#[derive(Serialize, Deserialize, Debug)]
pub struct ValueView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub created_at: Timestamp,
    pub label: String,
    /// Importance votes from members with Full standing when they voted.
    pub count: u32,
    pub average_x100: u32,
    /// The caller's latest vote, counted or not.
    pub mine: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImportanceInput {
    pub value: ActionHash,
    pub importance: u8,
}

/// web: "Organizer-controlled events don't use proposal values" (ADR 0061).
fn no_values_when_organized() -> ExternResult<()> {
    if project_integrity::organized()? {
        return Err(guest(
            "Organizer-controlled events don't use proposal values.",
        ));
    }
    Ok(())
}

pub fn add(label: String) -> ExternResult<ActionHash> {
    let membership =
        calls::my_membership()?.ok_or_else(|| guest("Join the project to add values."))?;
    no_values_when_organized()?;
    in_proposal_phase("Values can only be added during the proposal phase.")?;
    let label = label.trim().to_string();
    if label.is_empty() {
        return Err(guest("Write the value first."));
    }
    if label.chars().count() > LABEL_MAX {
        return Err(guest(format!("A value is at most {LABEL_MAX} characters.")));
    }
    let key = label_key(&label);
    if listed()?.iter().any(|v| label_key(&v.entry.label) == key) {
        return Err(guest("That value is already listed."));
    }
    let hash = create_entry(EntryTypes::Value(Value { label, membership }))?;
    create_link(space_scope::base()?, hash.clone(), LinkTypes::Values, ())?;
    Ok(hash)
}

pub fn rate(input: ImportanceInput) -> ExternResult<ActionHash> {
    let membership =
        calls::my_membership()?.ok_or_else(|| guest("Join the project to rate its values."))?;
    no_values_when_organized()?;
    in_proposal_phase("Values can only be voted on during the proposal phase.")?;
    if !importance_ok(input.importance) {
        return Err(guest("Importance is between 1 and 10."));
    }
    if !listed()?.iter().any(|v| v.hash == input.value) {
        return Err(guest("This value hasn't reached your node yet."));
    }
    let hash = create_entry(EntryTypes::Importance(Importance {
        value: input.value.clone(),
        membership,
        importance: input.importance,
    }))?;
    create_link(input.value, hash.clone(), LinkTypes::ValueToImportance, ())?;
    Ok(hash)
}

fn in_proposal_phase(refusal: &str) -> ExternResult<()> {
    let state = apply::state(&mut Standing::default(), &Roll::load()?)?;
    if state.phase() == Phase::P1 {
        Ok(())
    } else {
        Err(guest(refusal))
    }
}

/// Every value, oldest first; of two with the same label only the earlier counts.
pub fn listed() -> ExternResult<Vec<Found<Value>>> {
    let base = space_scope::base()?;
    let all = reads::linked::<Value>(base, LinkTypes::Values, UnitEntryTypes::Value)?;
    // Own values from other spaces come along with the own chain.
    let mut here = Vec::new();
    for v in all {
        if reads::in_space(&v.entry.membership)? {
            here.push(v);
        }
    }
    let mut all = here;
    all.sort_by(|a, b| (a.at, &a.hash).cmp(&(b.at, &b.hash)));
    let mut seen = Vec::new();
    all.retain(|v| {
        let key = label_key(&v.entry.label);
        let fresh = !seen.contains(&key);
        seen.push(key);
        fresh
    });
    Ok(all)
}

/// Every value with its importance, oldest first.
pub fn views(standing: &mut Standing) -> ExternResult<Vec<ValueView>> {
    let me = agent_info()?.agent_initial_pubkey;
    let mut out = Vec::new();
    for v in listed()? {
        let mut counted = Vec::new();
        let mut mine = None;
        for i in latest_votes(&v.hash)? {
            if i.author == me {
                mine = Some(i.entry.importance);
            }
            if standing.full(&i.author, i.at)? {
                counted.push(i.entry.importance);
            }
        }
        let summary = Summary::of(&counted);
        out.push(ValueView {
            id: v.hash,
            author: v.author,
            created_at: v.at,
            label: v.entry.label,
            count: summary.count,
            average_x100: summary.average_x100(),
            mine,
        });
    }
    Ok(out)
}

/// Values averaging 5 or more, as plan criteria: (criterion id, question).
pub fn prominent(views: &[ValueView]) -> Vec<(String, String)> {
    views
        .iter()
        .filter(|v| is_prominent(v.average_x100))
        .map(|v| (criterion_of(&v.id), criterion_label(&v.label)))
        .collect()
}

/// A value's plan criterion id.
pub fn criterion_of(value: &ActionHash) -> String {
    criterion_id(&ActionHashB64::from(value.clone()).to_string())
}

/// Each member's latest importance vote on `value`.
fn latest_votes(value: &ActionHash) -> ExternResult<Vec<Found<Importance>>> {
    let all = reads::linked::<Importance>(
        value.clone(),
        LinkTypes::ValueToImportance,
        UnitEntryTypes::Importance,
    )?;
    let mut latest: BTreeMap<AgentPubKey, Found<Importance>> = BTreeMap::new();
    for i in all.into_iter().filter(|i| i.entry.value == *value) {
        if latest.get(&i.author).is_none_or(|old| i.at > old.at) {
            latest.insert(i.author.clone(), i);
        }
    }
    Ok(latest.into_values().collect())
}
