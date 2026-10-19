//! The caller's own live links of one type, read from its source chain (visible at once).

use hdk::prelude::*;
use project_integrity::LinkTypes;

/// `(create action, first tag byte)` for each own link of `link_type` not yet deleted.
pub fn live_links(link_type: LinkTypes) -> ExternResult<Vec<(ActionHash, u8)>> {
    let scoped: ScopedLinkType = link_type.try_into()?;
    let actions = query(
        ChainQueryFilter::new()
            .action_type(ActionType::CreateLink)
            .include_entries(false),
    )?;
    let deletes = query(ChainQueryFilter::new().action_type(ActionType::DeleteLink))?;
    let deleted: Vec<ActionHash> = deletes
        .iter()
        .filter_map(|r| match &r.action().data {
            ActionData::DeleteLink(d) => Some(d.link_add_address.clone()),
            _ => None,
        })
        .collect();
    Ok(actions
        .iter()
        .filter_map(|r| match &r.action().data {
            ActionData::CreateLink(c)
                if c.zome_index == scoped.zome_index && c.link_type == scoped.zome_type =>
            {
                Some((
                    r.action_address().clone(),
                    c.tag.0.first().copied().unwrap_or(0),
                ))
            }
            _ => None,
        })
        .filter(|(hash, _)| !deleted.contains(hash))
        .collect())
}
