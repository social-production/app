//! Privacy switches (ADR 0112): display rules the device applies, stored as
//! `RequiresFollowApproval` self-links tagged with the switch's name (the untagged one is
//! follow approval). Everything stays public to the network; the app just doesn't show it.

use hdk::prelude::*;
use identity_integrity::LinkTypes;

use crate::follow_links::{edges, own_edges, Edge};

/// The switches web's Privacy section has, besides approval.
pub const SWITCHES: [&str; 3] = ["hide-profile", "hide-posts", "hide-from-feeds"];
const PREFIX: &str = "privacy:";

fn name_of(tag: &[u8]) -> Option<String> {
    let name = std::str::from_utf8(tag).ok()?.strip_prefix(PREFIX)?;
    SWITCHES.contains(&name).then(|| name.to_string())
}

/// The switch names among a person's approval self-links, sorted, once each.
pub fn switches(own: &[Edge]) -> Vec<String> {
    let mut names: Vec<String> = own.iter().filter_map(|e| name_of(&e.tag)).collect();
    names.sort();
    names.dedup();
    names
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SetPrivacy {
    pub name: String,
    pub on: bool,
}

/// Turn one of the caller's switches on or off; returns the caller's switches after.
#[hdk_extern]
pub fn set_privacy(input: SetPrivacy) -> ExternResult<Vec<String>> {
    if !SWITCHES.contains(&input.name.as_str()) {
        return Err(wasm_error!(WasmErrorInner::Guest(format!(
            "There is no privacy switch called {}.",
            input.name
        ))));
    }
    let me = agent_info()?.agent_initial_pubkey;
    let tag = format!("{PREFIX}{}", input.name).into_bytes();
    let current: Vec<Edge> = own_edges(LinkTypes::RequiresFollowApproval)?
        .into_iter()
        .filter(|e| e.tag == tag)
        .collect();
    if input.on && current.is_empty() {
        create_link(
            me.clone(),
            me,
            LinkTypes::RequiresFollowApproval,
            LinkTag::new(tag),
        )?;
    }
    if !input.on {
        for edge in current {
            delete_link(edge.hash, GetOptions::default())?;
        }
    }
    Ok(switches(&own_edges(LinkTypes::RequiresFollowApproval)?))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PrivacyOf {
    pub agent: AgentPubKey,
    pub privacy: Vec<String>,
}

/// The switches of each of `agents` (authors in a feed), skipping people with none on.
#[hdk_extern]
pub fn privacy_of(agents: Vec<AgentPubKey>) -> ExternResult<Vec<PrivacyOf>> {
    let mut out = Vec::new();
    for agent in agents.into_iter().take(200) {
        let privacy = switches(&edges(&agent, LinkTypes::RequiresFollowApproval)?);
        if !privacy.is_empty() {
            out.push(PrivacyOf { agent, privacy });
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::name_of;

    #[test]
    fn only_known_prefixed_names_count() {
        assert_eq!(
            name_of(b"privacy:hide-posts").as_deref(),
            Some("hide-posts")
        );
        assert_eq!(name_of(b""), None);
        assert_eq!(name_of(b"privacy:hide-everything"), None);
        assert_eq!(name_of(b"hide-posts"), None);
    }
}
