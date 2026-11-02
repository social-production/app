//! Reading report links (ADR 0071): the network's view merged with the caller's own chain, so
//! a report or vote shows at once on the device that wrote it.

use std::collections::BTreeMap;

use discussion_integrity::LinkTypes;
use hdk::prelude::*;

/// One link as reports read it.
#[derive(Clone, Debug)]
pub struct Seen {
    pub author: AgentPubKey,
    pub at: Timestamp,
    pub target: AnyLinkableHash,
    pub tag: Vec<u8>,
}

/// Every link of `link_type` on `base`, each once (reports' links are never deleted).
pub fn on(base: &ActionHash, link_type: LinkTypes) -> ExternResult<Vec<Seen>> {
    let mut found: BTreeMap<ActionHash, Seen> = BTreeMap::new();
    for link in get_links(
        LinkQuery::try_new(base.clone(), link_type)?,
        GetStrategy::default(),
    )? {
        found.insert(
            link.create_link_hash,
            Seen {
                author: link.author,
                at: link.timestamp,
                target: link.target,
                tag: link.tag.0,
            },
        );
    }
    let scoped: ScopedLinkType = link_type.try_into()?;
    let wanted = AnyLinkableHash::from(base.clone());
    for record in query(ChainQueryFilter::new().action_type(ActionType::CreateLink))? {
        let ActionData::CreateLink(c) = &record.action().data else {
            continue;
        };
        if c.zome_index != scoped.zome_index
            || c.link_type != scoped.zome_type
            || c.base_address != wanted
        {
            continue;
        }
        found
            .entry(record.action_address().clone())
            .or_insert_with(|| Seen {
                author: record.action().author().clone(),
                at: record.action().timestamp(),
                target: c.target_address.clone(),
                tag: c.tag.0.clone(),
            });
    }
    let mut out: Vec<Seen> = found.into_values().collect();
    out.sort_by_key(|s| s.at);
    Ok(out)
}
