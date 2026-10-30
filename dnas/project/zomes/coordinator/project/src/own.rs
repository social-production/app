//! The caller's own live links of one type in the current space, read from its source chain
//! (visible at once). In the commons a person has links in many spaces (ADR 0065).

use hdk::prelude::*;
use project_integrity::LinkTypes;

/// `(create action, first tag byte)` for each own link of `link_type` not yet deleted.
pub fn live_links(link_type: LinkTypes) -> ExternResult<Vec<(ActionHash, u8)>> {
    let base = space_scope::base()?;
    Ok(live_anywhere(link_type)?
        .into_iter()
        .filter(|(_, link)| link.base_address == base)
        .map(|(hash, link)| (hash, link.tag.0.first().copied().unwrap_or(0)))
        .collect())
}

/// `(create action, when)` for each own live link of `link_type` in the current space.
pub fn live_links_at(link_type: LinkTypes) -> ExternResult<Vec<(ActionHash, Timestamp)>> {
    let base = space_scope::base()?;
    Ok(live_timed(link_type)?
        .into_iter()
        .filter(|(_, link, _)| link.base_address == base)
        .map(|(hash, _, at)| (hash, at))
        .collect())
}

/// Each own live link of `link_type` in any space, with its create action.
pub fn live_anywhere(link_type: LinkTypes) -> ExternResult<Vec<(ActionHash, CreateLinkData)>> {
    Ok(live_timed(link_type)?
        .into_iter()
        .map(|(hash, link, _)| (hash, link))
        .collect())
}

fn live_timed(link_type: LinkTypes) -> ExternResult<Vec<(ActionHash, CreateLinkData, Timestamp)>> {
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
                    c.clone(),
                    r.action().timestamp(),
                ))
            }
            _ => None,
        })
        .filter(|(hash, _, _)| !deleted.contains(hash))
        .collect())
}
