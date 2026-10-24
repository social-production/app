//! The line a finished voice call leaves in a direct chat (web's `log_call`, ADR 0051). Either
//! phone may write it; the first per call shows, as web's server keeps the first.

use hdk::prelude::*;
use messages_integrity::{Call, EntryTypes, LinkTypes, UnitEntryTypes};
use sp_call::{check_call, first_per_call};
use sp_core::conversation::ChatKind;

use crate::entries::{all, Found};
use crate::people::{load, Chat};
use crate::{anchor, guest, MessageView, MESSAGES};

#[derive(Serialize, Deserialize, Debug)]
pub struct CallInput {
    pub call_id: String,
    /// "completed", "missed" or "declined".
    pub outcome: String,
    pub seconds: u32,
}

/// How a call row's line reads: web's `call_outcome` and `call_duration_seconds`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CallLine {
    pub outcome: String,
    pub seconds: u32,
}

fn logged(chat: &Chat) -> ExternResult<Vec<Found<Call>>> {
    if chat.kind != ChatKind::Direct {
        return Ok(Vec::new());
    }
    let calls = all::<Call>(UnitEntryTypes::Call, LinkTypes::Calls, MESSAGES)?;
    Ok(first_per_call(calls, |f| f.entry.call_id.as_str()))
}

/// web's `log_call`: add the call's line once; a second report of the same call is a no-op.
#[hdk_extern]
pub fn log_call(input: CallInput) -> ExternResult<()> {
    check_call(&input.call_id, &input.outcome, input.seconds).map_err(|reason| guest(&reason))?;
    let chat = load()?;
    if chat.kind != ChatKind::Direct {
        return Err(guest("Calls are only in direct chats."));
    }
    chat.require_member()?;
    if logged(&chat)?
        .iter()
        .any(|f| f.entry.call_id == input.call_id)
    {
        return Ok(());
    }
    let hash = create_entry(EntryTypes::Call(Call {
        call_id: input.call_id,
        outcome: input.outcome,
        seconds: input.seconds,
        invite: chat.my_invite()?,
    }))?;
    create_link(anchor()?, hash, LinkTypes::Calls, ())?;
    Ok(())
}

/// One chat row per logged call, by whoever logged it (web's `kind: "call"`, empty body).
pub fn rows(chat: &Chat) -> ExternResult<Vec<MessageView>> {
    Ok(logged(chat)?
        .into_iter()
        .map(|f| MessageView {
            id: f.hash,
            author: f.author,
            body: String::new(),
            created_at: f.at,
            reply_to: None,
            kind: "call".into(),
            edited: false,
            pinned: false,
            attachments: Vec::new(),
            call: Some(CallLine {
                outcome: f.entry.outcome,
                seconds: f.entry.seconds,
            }),
        })
        .collect())
}
