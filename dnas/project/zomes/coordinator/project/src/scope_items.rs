//! A private community's events (ADR 0069): each is a private clone of its own, listed here by
//! its seed so every member of the community finds it.

use hdk::prelude::*;
use project_integrity::LinkTypes;

/// One event listed on this community.
#[derive(Serialize, Deserialize, Debug)]
pub struct ScopeItem {
    pub seed: String,
    pub author: AgentPubKey,
    pub at: Timestamp,
}

/// List the caller's private event `seed` on the current community (once).
pub fn add(seed: String) -> ExternResult<()> {
    if list()?.iter().any(|i| i.seed == seed) {
        return Ok(());
    }
    // The tag carries the seed; the target is just who listed it.
    create_link(
        space_scope::base()?,
        agent_info()?.agent_initial_pubkey,
        LinkTypes::ScopeItems,
        LinkTag::new(seed.into_bytes()),
    )?;
    space_scope::touch()
}

/// The community's events, newest first, each seed once.
pub fn list() -> ExternResult<Vec<ScopeItem>> {
    let mut found = crate::links(LinkTypes::ScopeItems)?;
    found.sort_by_key(|l| std::cmp::Reverse(l.timestamp));
    let mut out: Vec<ScopeItem> = Vec::new();
    for link in found {
        let Ok(seed) = String::from_utf8(link.tag.0) else {
            continue;
        };
        if out.iter().all(|i| i.seed != seed) {
            out.push(ScopeItem {
                seed,
                author: link.author,
                at: link.timestamp,
            });
        }
    }
    Ok(out)
}
