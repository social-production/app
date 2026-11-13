//! Deactivating an account (ADR 0088): a `Deactivated` link from the account to itself, then
//! the chain is closed, so system validation refuses anything the account writes afterwards.
//! Readers show it as deactivated; clients refuse to sign it in.

use hdk::prelude::*;
use identity_integrity::LinkTypes;

use crate::records::links;

/// Deactivate the caller, for good. Its earlier posts, vouches and votes stay as they were.
pub fn deactivate() -> ExternResult<Timestamp> {
    let me = agent_info()?.agent_initial_pubkey;
    create_link(me.clone(), me, LinkTypes::Deactivated, ())?;
    close_chain(None)?;
    sys_time()
}

/// When `agent` deactivated its account, if it did (the caller's own from its chain).
pub fn since(agent: &AgentPubKey) -> ExternResult<Option<Timestamp>> {
    if *agent == agent_info()?.agent_initial_pubkey {
        let deactivated: ScopedLinkType = LinkTypes::Deactivated.try_into()?;
        let filter = ChainQueryFilter::new().action_type(ActionType::CreateLink);
        return Ok(query(filter)?.iter().find_map(|r| match &r.action().data {
            ActionData::CreateLink(l)
                if l.zome_index == deactivated.zome_index
                    && l.link_type == deactivated.zome_type =>
            {
                Some(r.action().timestamp())
            }
            _ => None,
        }));
    }
    Ok(links(agent.clone(), LinkTypes::Deactivated)?
        .into_iter()
        .filter(|l| l.author == *agent)
        .map(|l| l.timestamp)
        .min())
}
