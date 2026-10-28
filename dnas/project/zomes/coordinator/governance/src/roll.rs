//! The membership roll: every `Members` link with when it was made and, if its member left,
//! when it was deleted. Challenges are checked against it (ADR 0040).

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::LinkTypes;


pub struct Span {
    pub hash: ActionHash,
    pub agent: AgentPubKey,
    pub at: Timestamp,
    pub left: Option<Timestamp>,
}

impl Span {
    /// Joined before `at` and hadn't left by then.
    fn member_at(&self, at: Timestamp) -> bool {
        self.at < at && self.left.is_none_or(|left| left > at)
    }
}

pub struct Roll(Vec<Span>);

impl Roll {
    pub fn load() -> ExternResult<Self> {
        let base = space_scope::base()?;
        let details = get_links_details(
            LinkQuery::try_new(base, LinkTypes::Members)?,
            GetStrategy::default(),
        )?;
        let spans = details
            .into_inner()
            .into_iter()
            .map(|(create, deletes)| Span {
                hash: create.as_hash().clone(),
                agent: create.action().author().clone(),
                at: create.action().timestamp(),
                left: deletes.iter().map(|d| d.action().timestamp()).min(),
            })
            .collect();
        Ok(Self(spans))
    }

    /// Who made the membership link `hash`; asks the network when the roll hasn't seen it.
    pub fn agent(&self, hash: &ActionHash) -> ExternResult<Option<AgentPubKey>> {
        if let Some(span) = self.0.iter().find(|s| &s.hash == hash) {
            return Ok(Some(span.agent.clone()));
        }
        Ok(get(hash.clone(), GetOptions::default())?.map(|r| r.action().author().clone()))
    }

    /// The member of link `hash`, if it was a live membership at `at`.
    pub fn member_at(&self, hash: &ActionHash, at: Timestamp) -> Option<&AgentPubKey> {
        self.0
            .iter()
            .find(|s| &s.hash == hash && s.member_at(at))
            .map(|s| &s.agent)
    }

    /// Everyone who was a member at `at`, each with their earliest membership link then.
    pub fn members_at(&self, at: Timestamp) -> BTreeMap<AgentPubKey, ActionHash> {
        let mut earliest: BTreeMap<AgentPubKey, &Span> = BTreeMap::new();
        for span in self.0.iter().filter(|s| s.member_at(at)) {
            let entry = earliest.entry(span.agent.clone()).or_insert(span);
            if span.at < entry.at {
                *entry = span;
            }
        }
        earliest
            .into_iter()
            .map(|(agent, span)| (agent, span.hash.clone()))
            .collect()
    }
}
