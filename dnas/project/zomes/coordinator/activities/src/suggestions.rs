//! Role suggestions per activity (web's role suggestion and its decline).

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::{LinkTypes, RoleSuggestion, SuggestionDecline, UnitEntryTypes};

use crate::reads::{self, Found};

#[derive(Serialize, Deserialize, Debug)]
pub struct SuggestionView {
    pub id: ActionHash,
    pub person: AgentPubKey,
    pub by: AgentPubKey,
    pub at: Timestamp,
}

type PerRole<T> = BTreeMap<u8, T>;

/// The latest suggestion per role that its person hasn't declined, and every decline per role.
pub fn suggestions(
    activity: &ActionHash,
) -> ExternResult<(PerRole<SuggestionView>, PerRole<Vec<SuggestionView>>)> {
    let made: Vec<Found<RoleSuggestion>> = reads::linked::<RoleSuggestion>(
        activity.clone(),
        LinkTypes::ActivityToSuggestions,
        UnitEntryTypes::RoleSuggestion,
    )?
    .into_iter()
    .filter(|s| s.entry.activity == *activity)
    .collect();
    let declines = reads::linked::<SuggestionDecline>(
        activity.clone(),
        LinkTypes::ActivityToSuggestions,
        UnitEntryTypes::SuggestionDecline,
    )?;
    let mut declined: PerRole<Vec<SuggestionView>> = BTreeMap::new();
    for d in &declines {
        let Some(s) = made
            .iter()
            .find(|s| s.hash == d.entry.suggestion && s.entry.person == d.author)
        else {
            continue;
        };
        declined
            .entry(s.entry.role)
            .or_default()
            .push(SuggestionView {
                id: d.hash.clone(),
                person: d.author.clone(),
                by: s.author.clone(),
                at: d.at,
            });
    }
    let mut latest: PerRole<&Found<RoleSuggestion>> = BTreeMap::new();
    for s in &made {
        latest.insert(s.entry.role, s);
    }
    let standing = latest
        .into_iter()
        .filter(|(_, s)| {
            !declines
                .iter()
                .any(|d| d.entry.suggestion == s.hash && d.author == s.entry.person)
        })
        .map(|(role, s)| {
            (
                role,
                SuggestionView {
                    id: s.hash.clone(),
                    person: s.entry.person.clone(),
                    by: s.author.clone(),
                    at: s.at,
                },
            )
        })
        .collect();
    Ok((standing, declined))
}
