//! What web's bot review notifications are made from (ADR 0074): the openings and closings of
//! your own review, and of reviews of accounts you marked or voted on. web also tells full
//! members who share a space with the account; that needs every space's members and is left
//! out here.

use std::collections::BTreeMap;

use hdk::prelude::*;
use identity_integrity::LinkTypes;
use sp_bot_review::{Logged, APPEAL_AFTER_US};

use crate::bot_review::state;

#[derive(Serialize, Deserialize, Debug)]
pub struct BotNotice {
    pub account: AgentPubKey,
    /// "opened", "upheld" or "cleared".
    pub kind: String,
    pub is_appeal: bool,
    pub at: Timestamp,
    /// Upheld: when a re-review may be asked for (web's body names the date).
    pub re_review_from: Option<Timestamp>,
}

/// Accounts the caller marked or voted on, with the time of its first such link.
fn my_reviewed_accounts() -> ExternResult<BTreeMap<AgentPubKey, i64>> {
    let types: Vec<ScopedLinkType> = [LinkTypes::BotMarks, LinkTypes::BotReviewVotes]
        .into_iter()
        .map(ScopedLinkType::try_from)
        .collect::<Result<_, _>>()?;
    let filter = ChainQueryFilter::new().action_type(ActionType::CreateLink);
    let mut out = BTreeMap::new();
    for record in query(filter)? {
        let ActionData::CreateLink(link) = &record.action().data else {
            continue;
        };
        let ours = types
            .iter()
            .any(|t| t.zome_index == link.zome_index && t.zome_type == link.link_type);
        if let (true, Some(account)) = (ours, link.base_address.clone().into_agent_pub_key()) {
            out.entry(account)
                .or_insert(record.action().timestamp().as_micros());
        }
    }
    Ok(out)
}

fn notices_of(account: &AgentPubKey, since: i64, own: bool) -> ExternResult<Vec<BotNotice>> {
    let Some(s) = state(account)? else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for logged in s.log {
        let (kind, at, is_appeal) = match logged {
            // web tells nobody when a re-review opens (only `maybe_open_review` notifies).
            Logged::Opened {
                is_appeal: true, ..
            } => continue,
            Logged::Opened { at, is_appeal } => ("opened", at, is_appeal),
            Logged::Closed(c) if c.upheld => ("upheld", c.at, c.is_appeal),
            Logged::Closed(c) => ("cleared", c.at, c.is_appeal),
        };
        if !own && at < since {
            continue;
        }
        out.push(BotNotice {
            account: account.clone(),
            kind: kind.into(),
            is_appeal,
            at: Timestamp(at),
            re_review_from: (kind == "upheld").then_some(Timestamp(at + APPEAL_AFTER_US)),
        });
    }
    Ok(out)
}

pub fn mine() -> ExternResult<Vec<BotNotice>> {
    let me = agent_info()?.agent_initial_pubkey;
    let mut out = notices_of(&me, i64::MIN, true)?;
    for (account, since) in my_reviewed_accounts()? {
        if account != me {
            out.extend(notices_of(&account, since, false)?);
        }
    }
    Ok(out)
}
