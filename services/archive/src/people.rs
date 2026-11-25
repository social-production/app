//! People search for web's Search page (`/index/people?q=`): the archivist reads every
//! profile in the directory now and then and keeps names and bios here, so guests can search
//! people too without listing the directory themselves. Ranking follows web-backend's
//! `search.py`: exact name, then prefix, then anywhere in the name or bio.

use std::sync::Arc;

use holo_hash::AgentPubKeyB64;
use holochain_types::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::Calls;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonCard {
    pub agent: String,
    pub name: String,
    pub bio: String,
    /// A bot review found the account is a bot (ADR 0074; web's `pausedUsernames`).
    #[serde(skip)]
    pub paused: bool,
}

/// The part of `get_standing` the index reads.
#[derive(Debug, Deserialize)]
struct Standing {
    level: String,
}

/// The profile fields search needs (mirrors the identity zome's `Profile`).
#[derive(Debug, Deserialize)]
struct Profile {
    name: String,
    bio: String,
}

#[derive(Clone, Debug, Default)]
pub struct PeopleIndex(Arc<RwLock<Vec<PersonCard>>>);

impl PeopleIndex {
    /// People whose name or bio contains `query` (case-insensitive), best match first.
    pub async fn search(&self, query: &str, limit: usize) -> Vec<PersonCard> {
        rank(&self.0.read().await, query, limit)
    }

    /// Everyone a bot review paused, for web's "Paused" badge on author lines.
    pub async fn paused(&self) -> Vec<String> {
        let people = self.0.read().await;
        people
            .iter()
            .filter(|p| p.paused)
            .map(|p| p.agent.clone())
            .collect()
    }
}

fn rank(people: &[PersonCard], query: &str, limit: usize) -> Vec<PersonCard> {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return Vec::new();
    }
    let mut found: Vec<(u8, &PersonCard)> = people
        .iter()
        .filter_map(|p| {
            let name = p.name.to_lowercase();
            let score = if name == needle {
                0
            } else if name.starts_with(&needle) {
                1
            } else if name.contains(&needle) || p.bio.to_lowercase().contains(&needle) {
                2
            } else {
                return None;
            };
            Some((score, p))
        })
        .collect();
    found.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()))
    });
    found
        .into_iter()
        .take(limit)
        .map(|(_, p)| p.clone())
        .collect()
}

/// Re-read every profile in the directory into `people`. Returns how many.
pub async fn people_round(
    calls: &impl Calls,
    directory: &CellId,
    people: &PeopleIndex,
) -> anyhow::Result<usize> {
    let keys: Vec<AgentPubKey> = calls.call(directory, "identity", "list_people", ()).await?;
    let mut found = Vec::with_capacity(keys.len());
    for key in keys {
        let profile: Option<Profile> = calls
            .call(directory, "identity", "get_profile", key.clone())
            .await
            .unwrap_or(None);
        let Some(p) = profile else { continue };
        let standing: Option<Standing> = calls
            .call(directory, "identity", "get_standing", key.clone())
            .await
            .ok();
        found.push(PersonCard {
            agent: AgentPubKeyB64::from(key).to_string(),
            name: p.name,
            bio: p.bio,
            paused: standing.is_some_and(|s| s.level == "Paused"),
        });
    }
    let count = found.len();
    *people.0.write().await = found;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::{rank, PersonCard};

    fn person(name: &str, bio: &str) -> PersonCard {
        PersonCard {
            agent: name.into(),
            name: name.into(),
            bio: bio.into(),
            paused: false,
        }
    }

    #[test]
    fn exact_then_prefix_then_anywhere() {
        let people = [
            person("annabel", ""),
            person("joann", ""),
            person("ann", ""),
            person("bob", "friends with Ann"),
            person("carl", ""),
        ];
        let names: Vec<String> = rank(&people, "Ann", 10)
            .into_iter()
            .map(|p| p.name)
            .collect();
        assert_eq!(names, ["ann", "annabel", "bob", "joann"]);
        assert!(rank(&people, "  ", 10).is_empty());
        assert_eq!(rank(&people, "ann", 2).len(), 2);
    }
}
