//! Founders (ADR pending, docs/plans/M1-identity.md): agents that start with Full standing
//! so the vouch chain can begin. Listed in `dnas/directory/dna.yaml` properties, so changing
//! the list changes the DNA hash, which means a new network.

use hdi::prelude::*;
use holo_hash::AgentPubKeyB64;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Founder {
    pub name: String,
    /// Base64 agent key, e.g. "uhCAk…".
    pub agent: String,
}

#[dna_properties]
pub struct DirectoryProperties {
    pub kind: String,
    #[serde(default)]
    pub founders: Vec<Founder>,
}

/// Founder agent keys. A malformed key in dna.yaml is a build error, not a silent skip.
pub fn founder_agents() -> ExternResult<Vec<AgentPubKey>> {
    DirectoryProperties::try_from_dna_properties()?
        .founders
        .iter()
        .map(|f| {
            AgentPubKeyB64::from_b64_str(&f.agent)
                .map(AgentPubKey::from)
                .map_err(|e| wasm_error!(WasmErrorInner::Guest(format!("founder {}: {e}", f.name))))
        })
        .collect()
}

pub fn is_founder(agent: &AgentPubKey) -> ExternResult<bool> {
    Ok(founder_agents()?.contains(agent))
}
