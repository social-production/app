//! People search for web's Search page (`/index/people?q=`): the archivist reads every
//! profile in the directory now and then and keeps names and bios here, so guests can search
//! people too without listing the directory themselves. Ranking follows web-backend's
//! `search.py`: exact name, then prefix, then anywhere in the name or bio.

use std::path::Path;
use std::sync::Arc;

use holo_hash::AgentPubKeyB64;
use holochain::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use super::archivist::provisioned;
use super::Node;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonCard {
    pub agent: String,
    pub name: String,
    pub bio: String,
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

impl Node {
    /// Re-read every profile in the directory into the people index. Returns how many.
    pub async fn people_round(&self, happ: &Path) -> anyhow::Result<usize> {
        let app = self.archivist_app(happ).await?;
        let agent = app.agent_pub_key.clone();
        let directory = provisioned(&app, "directory")?;
        let keys: Vec<AgentPubKey> = self
            .call(&agent, directory.clone(), "identity", "list_people", ())
            .await?;
        let mut people = Vec::with_capacity(keys.len());
        for key in keys {
            let profile: Option<Profile> = self
                .call(
                    &agent,
                    directory.clone(),
                    "identity",
                    "get_profile",
                    key.clone(),
                )
                .await
                .unwrap_or(None);
            if let Some(p) = profile {
                people.push(PersonCard {
                    agent: AgentPubKeyB64::from(key).to_string(),
                    name: p.name,
                    bio: p.bio,
                });
            }
        }
        let count = people.len();
        *self.people.0.write().await = people;
        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::{rank, PersonCard};

    fn person(name: &str, bio: &str) -> PersonCard {
        PersonCard {
            agent: name.into(),
            name: name.into(),
            bio: bio.into(),
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
