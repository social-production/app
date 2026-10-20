//! The inbox (the base cell of the conversation role, ADR 0043): an invitation is a link on the
//! invited person's key whose tag is the chat's seed, encrypted from sender to recipient. Only
//! the recipient can read which chat it is; who invited whom is visible to the network.

use std::collections::BTreeSet;

use hdk::prelude::*;
use messages_integrity::{network, LinkTypes, Network};
use sp_core::conversation::{chat_seed, INBOX_TAG_MAX};

use crate::guest;

#[derive(Serialize, Deserialize, Debug)]
pub struct InviteInput {
    pub to: AgentPubKey,
    /// The chat clone's network seed.
    pub seed: String,
}

/// A chat someone invited you to (you included: your own chats are in your inbox too).
#[derive(Serialize, Deserialize, Debug)]
pub struct Invitation {
    pub from: AgentPubKey,
    pub seed: String,
    pub at: Timestamp,
}

fn inbox_only() -> ExternResult<()> {
    match network()? {
        Network::Inbox => Ok(()),
        Network::Chat(..) => Err(guest("Invitations go through the inbox cell.")),
    }
}

/// Tell `to` about a chat. Its seed reaches only them; send one to yourself too, so your
/// other devices and a restored account find the chat.
#[hdk_extern]
pub fn send_invite(input: InviteInput) -> ExternResult<()> {
    inbox_only()?;
    if chat_seed(&input.seed).is_none() {
        return Err(guest("That isn't a chat."));
    }
    let me = agent_info()?.agent_initial_pubkey;
    let sealed = ed_25519_x_salsa20_poly1305_encrypt(
        me.clone(),
        input.to.clone(),
        input.seed.into_bytes().into(),
    )?;
    let tag = ExternIO::encode(sealed).map_err(|e| wasm_error!(e))?.0;
    if tag.len() > INBOX_TAG_MAX {
        return Err(guest("That chat's name is too long to send."));
    }
    create_link(input.to, me, LinkTypes::Inbox, LinkTag(tag))?;
    Ok(())
}

fn open(me: &AgentPubKey, from: &AgentPubKey, tag: &[u8]) -> Option<String> {
    let sealed: XSalsa20Poly1305EncryptedData = ExternIO(tag.to_vec()).decode().ok()?;
    let data = ed_25519_x_salsa20_poly1305_decrypt(me.clone(), from.clone(), sealed).ok()?;
    let seed = String::from_utf8(data.as_ref().to_vec()).ok()?;
    chat_seed(&seed).is_some().then_some(seed)
}

/// Every chat you were invited to, oldest first, each once. Invitations that don't open
/// (not for you, or damaged) are skipped.
#[hdk_extern]
pub fn inbox() -> ExternResult<Vec<Invitation>> {
    inbox_only()?;
    let me = agent_info()?.agent_initial_pubkey;
    let mut found: Vec<(AgentPubKey, Vec<u8>, Timestamp)> = own_links_to(&me)?;
    let links = get_links(
        LinkQuery::try_new(me.clone(), LinkTypes::Inbox)?,
        GetStrategy::default(),
    )?;
    found.extend(links.into_iter().map(|l| (l.author, l.tag.0, l.timestamp)));
    found.sort_by_key(|(_, _, at)| *at);
    let mut seen = BTreeSet::new();
    Ok(found
        .into_iter()
        .filter_map(|(from, tag, at)| {
            let seed = open(&me, &from, &tag)?;
            seen.insert(seed.clone())
                .then_some(Invitation { from, seed, at })
        })
        .collect())
}

/// Invitations you sent yourself, from your own chain (shown before gossip).
fn own_links_to(me: &AgentPubKey) -> ExternResult<Vec<(AgentPubKey, Vec<u8>, Timestamp)>> {
    let scoped: ScopedLinkType = LinkTypes::Inbox.try_into()?;
    let base = AnyLinkableHash::from(me.clone());
    let creates = query(ChainQueryFilter::new().action_type(ActionType::CreateLink))?;
    Ok(creates
        .iter()
        .filter_map(|r| match &r.action().data {
            ActionData::CreateLink(c)
                if c.zome_index == scoped.zome_index
                    && c.link_type == scoped.zome_type
                    && c.base_address == base =>
            {
                Some((me.clone(), c.tag.0.clone(), r.action().timestamp()))
            }
            _ => None,
        })
        .collect())
}
