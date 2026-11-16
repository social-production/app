//! Who an event's invitations went to (ADR 0111). Sending an inbox invite (conversation DNA)
//! also writes an `Invite` record here, so every organizer sees the whole Invited list with
//! who invited each person and when; the seed itself still reaches only the invitee.

use hdk::prelude::*;
use project_integrity::{audience, EntryTypes, Invite, LinkTypes};
use space_scope::{enter, In};

use crate::{links, organizers};

/// One invitation: who was invited, by whom, and when (newest record per person wins).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InviteView {
    pub agent: AgentPubKey,
    pub by: AgentPubKey,
    pub at: Timestamp,
}

/// Record that you invited `agent` (after sending the inbox invite). Once per person.
#[hdk_extern]
pub fn record_invite(call: In<AgentPubKey>) -> ExternResult<()> {
    let agent = enter(call)?;
    let me = agent_info()?.agent_initial_pubkey;
    if agent == me || list()?.iter().any(|i| i.agent == agent && i.by == me) {
        return Ok(());
    }
    let grant = if audience()?.is_private() {
        let role = organizers::role_of(&me)?.ok_or_else(|| {
            wasm_error!(WasmErrorInner::Guest(
                "Only organizers can invite people to a private event.".into()
            ))
        })?;
        role.grant()
    } else {
        None
    };
    let hash = create_entry(EntryTypes::Invite(Invite { agent, grant }))?;
    create_link(space_scope::base()?, hash, LinkTypes::Invites, ())?;
    space_scope::touch()
}

/// Every invitation recorded for this event, oldest first, one per person (the latest).
/// Invites a removed organizer wrote after their removal are left out (ADR 0073).
#[hdk_extern]
pub fn event_invites(call: In<()>) -> ExternResult<Vec<InviteView>> {
    enter(call)?;
    list()
}

fn list() -> ExternResult<Vec<InviteView>> {
    let ended = organizers::ledger()?.ended;
    let mut found = links(LinkTypes::Invites)?;
    found.sort_by_key(|l| l.timestamp);
    let mut out: Vec<InviteView> = Vec::new();
    for link in found {
        let Some(hash) = link.target.into_action_hash() else {
            continue;
        };
        let Some(record) = get(hash, GetOptions::default())? else {
            continue;
        };
        let Ok(Some(invite)) = record.entry().to_app_option::<Invite>() else {
            continue;
        };
        let at = record.action().timestamp();
        if invite
            .grant
            .as_ref()
            .and_then(|g| ended.get(g))
            .is_some_and(|end| *end <= at)
        {
            continue;
        }
        out.retain(|i| i.agent != invite.agent);
        out.push(InviteView {
            agent: invite.agent,
            by: record.action().author().clone(),
            at,
        });
    }
    Ok(out)
}
