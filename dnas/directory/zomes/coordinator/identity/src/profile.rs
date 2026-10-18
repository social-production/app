use hdk::prelude::*;
use identity_integrity::{Avatar, EntryTypes, LinkTypes, Profile};

use crate::records::{action_targets, decode, latest, links};

const ALL_PEOPLE: &str = "all_people";

#[derive(Serialize, Deserialize, Debug)]
pub struct ProfileInput {
    pub name: String,
    pub bio: String,
    pub home_area: String,
    pub avatar: Option<Avatar>,
}

/// Create the caller's profile, or update it if one exists.
pub fn set(input: ProfileInput) -> ExternResult<ActionHash> {
    let me = agent_info()?.agent_initial_pubkey;
    let profile = Profile {
        name: input.name.trim().to_string(),
        bio: input.bio,
        home_area: input.home_area.trim().to_string(),
        avatar: input.avatar,
    };
    if let Some(original) = original_of(&me)? {
        return update_entry(original, &EntryTypes::Profile(profile));
    }
    let hash = create_entry(&EntryTypes::Profile(profile))?;
    create_link(me.clone(), hash.clone(), LinkTypes::AgentToProfile, ())?;
    let anchor = Path::from(ALL_PEOPLE).path_entry_hash()?;
    create_link(anchor, me, LinkTypes::AllPeople, ())?;
    Ok(hash)
}

pub fn get(agent: &AgentPubKey) -> ExternResult<Option<Profile>> {
    let Some(original) = original_of(agent)? else {
        return Ok(None);
    };
    match latest(original)? {
        Some(record) => decode::<Profile>(&record),
        None => Ok(None),
    }
}

/// Everyone who has written a profile, oldest first.
pub fn list_people() -> ExternResult<Vec<AgentPubKey>> {
    let anchor = Path::from(ALL_PEOPLE).path_entry_hash()?;
    let mut people = links(anchor, LinkTypes::AllPeople)?;
    people.sort_by_key(|l| l.timestamp);
    let mut seen = Vec::new();
    for agent in people
        .into_iter()
        .filter_map(|l| l.target.into_agent_pub_key())
    {
        if !seen.contains(&agent) {
            seen.push(agent);
        }
    }
    Ok(seen)
}

/// The first profile the agent created (later versions are updates of it).
fn original_of(agent: &AgentPubKey) -> ExternResult<Option<ActionHash>> {
    let mut found = links(agent.clone(), LinkTypes::AgentToProfile)?;
    found.sort_by_key(|l| l.timestamp);
    Ok(action_targets(found).into_iter().next())
}
