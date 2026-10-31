//! Guest reads (ADR 0052): signed-out people open project pages, as on web. The
//! archivist answers a short list of read-only calls on the directory and on the public
//! spaces in its commons cell (ADR 0065). Chats (the conversation DNA) and anything about devices are never served.

use anyhow::anyhow;
use serde::Deserialize;

use super::archivist::provisioned;
use super::spaces::{space_of_seed, Scoped};
use super::Node;

/// Read-only calls a guest may make, by zome. Every one is public to any member of the
/// network anyway: listings, profiles, standing, follows and feedback in the directory; a project's page,
/// proposals, plans, values, links and chat in its own network.
const DIRECTORY: &[(&str, &[&str])] = &[(
    "identity",
    &[
        "get_profile",
        "get_standing",
        "get_listing",
        "find_listing",
        "follow_view",
        "list_feedback",
        "get_feedback",
    ],
)];
const PROJECT: &[(&str, &[&str])] = &[
    ("project", &["get_project"]),
    (
        "governance",
        &[
            "list_proposals",
            "list_plans",
            "list_values",
            "link_sides",
            "list_links",
        ],
    ),
    ("activities", &["list_activities"]),
    (
        "discussion",
        &["list_messages", "get_chunk", "message_votes"],
    ),
];

/// Calls not about one space (file parts are found by their own hash).
const UNSCOPED: &[&str] = &["get_chunk"];

/// One guest call: `seed` names the project (none means the directory).
#[derive(Debug, Deserialize)]
pub struct PublicRead {
    pub seed: Option<String>,
    pub zome: String,
    #[serde(rename = "fn")]
    pub func: String,
    /// The call's msgpack payload, as `@holochain/client` would send it.
    #[serde(default)]
    pub payload: Vec<u8>,
}

/// Why a guest read was refused (the route turns these into 403 / 404).
#[derive(Debug)]
pub enum Refused {
    NotAllowed,
    NotHere,
}

impl Node {
    /// Answer a guest's read as the archivist; the result is msgpack, like a zome call's.
    pub async fn public_read(&self, read: PublicRead) -> anyhow::Result<Result<Vec<u8>, Refused>> {
        let table = if read.seed.is_some() {
            PROJECT
        } else {
            DIRECTORY
        };
        if !allowed(table, &read.zome, &read.func) {
            return Ok(Err(Refused::NotAllowed));
        }
        let id = format!("archivist-{}", self.happ_version);
        let Some(app) = self.find_app(&id).await? else {
            return Ok(Err(Refused::NotHere));
        };
        let space = match &read.seed {
            None => None,
            // Private spaces are never served; only listed public ones.
            Some(seed) => match space_of_seed(seed) {
                Some(space) => Some(space),
                None => return Ok(Err(Refused::NotHere)),
            },
        };
        let payload: rmpv::Value = if read.payload.is_empty() {
            rmpv::Value::Nil
        } else {
            rmpv::decode::read_value(&mut read.payload.as_slice())
                .map_err(|e| anyhow!("payload: {e}"))?
        };
        let agent = app.agent_pub_key.clone();
        let out: rmpv::Value = match space {
            None => {
                let directory = provisioned(&app, "directory")?;
                self.call(&agent, directory, &read.zome, &read.func, payload)
                    .await?
            }
            Some(space) => {
                let commons = provisioned(&app, "project")?;
                if UNSCOPED.contains(&read.func.as_str()) {
                    self.call(&agent, commons, &read.zome, &read.func, payload)
                        .await?
                } else {
                    let input = Scoped {
                        space,
                        input: payload,
                    };
                    self.call(&agent, commons, &read.zome, &read.func, input)
                        .await?
                }
            }
        };
        let mut bytes = Vec::new();
        rmpv::encode::write_value(&mut bytes, &out).map_err(|e| anyhow!("result: {e}"))?;
        Ok(Ok(bytes))
    }
}

fn allowed(table: &[(&str, &[&str])], zome: &str, func: &str) -> bool {
    table
        .iter()
        .any(|(z, fns)| *z == zome && fns.contains(&func))
}

#[cfg(test)]
mod tests {
    use super::{allowed, DIRECTORY, PROJECT};

    #[test]
    fn only_listed_reads() {
        assert!(allowed(PROJECT, "project", "get_project"));
        assert!(allowed(PROJECT, "discussion", "list_messages"));
        assert!(allowed(PROJECT, "activities", "list_activities"));
        assert!(!allowed(PROJECT, "activities", "take_role"));
        assert!(allowed(DIRECTORY, "identity", "get_profile"));
        assert!(!allowed(PROJECT, "project", "join"));
        assert!(!allowed(PROJECT, "identity", "get_profile"));
        assert!(!allowed(DIRECTORY, "identity", "get_devices"));
        assert!(!allowed(DIRECTORY, "messages", "list_messages"));
    }
}
