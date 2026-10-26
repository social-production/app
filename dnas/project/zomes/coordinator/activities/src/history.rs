//! Past activity (web's ActivityHistorySection): participants check in whether an activity
//! happened once it has ended, as web-backend's history completion does for project activities.

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::{ActivityCompletion, EntryTypes, LinkTypes, UnitEntryTypes};
use sp_activity::roles_open;

use crate::reads::{self, Found};
use crate::view;
use crate::write::guest;

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckInInput {
    pub activity: ActionHash,
    /// Completed, uncompleted, or `None` to clear your check-in.
    pub done: Option<bool>,
}

/// web's participant side: who could check in, and how they did.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct CompletionView {
    pub eligible: u32,
    pub completed: u32,
    pub uncompleted: u32,
    /// The caller's check-in, if they are eligible and have one.
    pub mine: Option<bool>,
    pub can_set: bool,
}

pub fn check_in(input: CheckInInput) -> ExternResult<ActionHash> {
    let found = reads::found::<project_integrity::Activity>(input.activity.clone())?
        .ok_or_else(|| guest("This activity hasn't reached your node yet."))?;
    if roles_open(found.entry.ends_at.as_micros(), sys_time()?.as_micros()) {
        return Err(guest("Activity has not ended yet"));
    }
    let me = agent_info()?.agent_initial_pubkey;
    let Some(assignment) = view::my_take(&input.activity, &me)? else {
        return Err(guest(
            "Only assigned participants can set participant completion",
        ));
    };
    let hash = create_entry(EntryTypes::ActivityCompletion(ActivityCompletion {
        activity: input.activity.clone(),
        done: input.done,
        assignment,
    }))?;
    create_link(
        input.activity,
        hash.clone(),
        LinkTypes::ActivityToRatings,
        (),
    )?;
    Ok(hash)
}

/// The participants' check-ins among those who held a role (`holders`).
pub fn completion(
    activity: &ActionHash,
    holders: &[AgentPubKey],
    me: &AgentPubKey,
) -> ExternResult<CompletionView> {
    let all = reads::linked::<ActivityCompletion>(
        activity.clone(),
        LinkTypes::ActivityToRatings,
        UnitEntryTypes::ActivityCompletion,
    )?;
    let mut latest: BTreeMap<AgentPubKey, Found<ActivityCompletion>> = BTreeMap::new();
    for c in all.into_iter().filter(|c| c.entry.activity == *activity) {
        latest.insert(c.author.clone(), c);
    }
    let state = |agent: &AgentPubKey| latest.get(agent).and_then(|c| c.entry.done);
    let count = |want: bool| {
        let n = holders.iter().filter(|a| state(a) == Some(want)).count();
        u32::try_from(n).unwrap_or(u32::MAX)
    };
    let can_set = holders.contains(me);
    Ok(CompletionView {
        eligible: u32::try_from(holders.len()).unwrap_or(u32::MAX),
        completed: count(true),
        uncompleted: count(false),
        mine: if can_set { state(me) } else { None },
        can_set,
    })
}
