//! Who is in the chat and who manages it, replayed from its invitations, removals and leaves
//! (`sp_core::roster`, ADR 0045), and the citations the caller's writes need.

use hdk::prelude::*;
use messages_integrity::{network, Invite, Leave, LinkTypes, Network, Removal, UnitEntryTypes};
use sp_core::conversation::ChatKind;
use sp_core::roster::{Change, Roster};

use crate::entries::{all, Found};
use crate::guest;

/// Where invitations, removals, leaves and names are listed.
pub const MEMBERS: &str = "members";

pub fn chat() -> ExternResult<(AgentPubKey, ChatKind)> {
    match network()? {
        Network::Chat(creator, kind) => Ok((creator, kind)),
        Network::Inbox => Err(guest("This is the inbox, not a chat.")),
    }
}

pub struct Chat {
    pub creator: AgentPubKey,
    pub kind: ChatKind,
    pub me: AgentPubKey,
    pub roster: Roster<AgentPubKey, Timestamp>,
    pub invites: Vec<Found<Invite>>,
    pub leaves: Vec<Found<Leave>>,
}

pub fn load() -> ExternResult<Chat> {
    let (creator, kind) = chat()?;
    let invites: Vec<Found<Invite>> = all(UnitEntryTypes::Invite, LinkTypes::Invites, MEMBERS)?;
    let removals: Vec<Found<Removal>> = all(UnitEntryTypes::Removal, LinkTypes::Removals, MEMBERS)?;
    let leaves: Vec<Found<Leave>> = all(UnitEntryTypes::Leave, LinkTypes::Leaves, MEMBERS)?;
    let mut changes: Vec<(Timestamp, ActionHash, Change<AgentPubKey>)> = Vec::new();
    for f in &invites {
        let change = Change::Invite {
            by: f.author.clone(),
            agent: f.entry.agent.clone(),
        };
        changes.push((f.at, f.hash.clone(), change));
    }
    for f in &removals {
        let change = Change::Remove {
            by: f.author.clone(),
            agent: f.entry.agent.clone(),
        };
        changes.push((f.at, f.hash.clone(), change));
    }
    for f in &leaves {
        let change = Change::Leave {
            agent: f.author.clone(),
            successor: f.entry.successor.clone(),
        };
        changes.push((f.at, f.hash.clone(), change));
    }
    changes.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
    let timeline: Vec<(Timestamp, Change<AgentPubKey>)> =
        changes.into_iter().map(|(at, _, c)| (at, c)).collect();
    Ok(Chat {
        roster: Roster::replay(creator.clone(), &timeline),
        creator,
        kind,
        me: agent_info()?.agent_initial_pubkey,
        invites,
        leaves,
    })
}

impl Chat {
    pub fn is_member(&self) -> bool {
        self.roster.members.contains(&self.me)
    }

    pub fn is_manager(&self) -> bool {
        self.roster.manager.as_ref() == Some(&self.me)
    }

    /// The caller's newest invitation, which their messages cite; `None` for the creator.
    pub fn my_invite(&self) -> ExternResult<Option<ActionHash>> {
        if self.me == self.creator {
            return Ok(None);
        }
        self.invites
            .iter()
            .rev()
            .find(|f| f.entry.agent == self.me)
            .map(|f| Some(f.hash.clone()))
            .ok_or_else(|| {
                guest("Your invitation to this chat hasn't arrived yet. Try again in a moment.")
            })
    }

    /// How the caller manages the group: `None` as its creator, else the `Leave` that handed
    /// it to them.
    pub fn my_authority(&self) -> ExternResult<Option<ActionHash>> {
        if !self.is_manager() {
            return Err(guest("Only the group creator can manage members"));
        }
        if self.me == self.creator && self.handed_over_by_me().is_none() {
            return Ok(None);
        }
        self.leaves
            .iter()
            .rev()
            .find(|f| f.entry.successor.as_ref() == Some(&self.me))
            .map(|f| Some(f.hash.clone()))
            .ok_or_else(|| guest("The group's handover hasn't arrived yet. Try again in a moment."))
    }

    fn handed_over_by_me(&self) -> Option<&Found<Leave>> {
        self.leaves
            .iter()
            .find(|f| f.author == self.me && f.entry.successor.is_some())
    }

    /// What the caller cites to pin: either person in a direct chat, the manager in a group.
    pub fn pin_authority(&self) -> ExternResult<Option<ActionHash>> {
        match self.kind {
            ChatKind::Direct => self.my_invite(),
            ChatKind::Group => self
                .my_authority()
                .map_err(|_| guest("Only the group creator can pin messages")),
        }
    }

    /// Writes from the caller need them to be in the chat now.
    pub fn require_member(&self) -> ExternResult<()> {
        if self.is_member() {
            Ok(())
        } else {
            Err(guest("You're not in this chat anymore."))
        }
    }

    pub fn require_group(&self) -> ExternResult<()> {
        if self.kind == ChatKind::Group {
            Ok(())
        } else {
            Err(guest("Conversation is not a group"))
        }
    }
}
