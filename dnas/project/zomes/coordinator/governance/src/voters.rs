//! Who a vote is counted against (ADRs 0033, 0118, 0121): the space's members who were active
//! in the seven days before it opened (web's `weekly_active_project_members`), leaving out
//! accounts found to be bots; the proposer always counts.

use std::collections::BTreeSet;

use hdk::prelude::*;

use crate::calls;
use crate::decide::Standing;
use crate::roll::Roll;

/// The membership links a proposal opened now records as its population. With `votes` off
/// (organizers decide, ADR 0061) it is every member, as nobody's vote is counted.
pub fn snapshot(
    roll: &Roll,
    proposer: Option<&ActionHash>,
    now: Timestamp,
    standing: &mut Standing,
    votes: bool,
) -> ExternResult<Vec<ActionHash>> {
    let all = calls::member_snapshot()?;
    if !votes {
        return Ok(all);
    }
    let mut agents = Vec::new();
    for hash in &all {
        agents.extend(roll.agent(hash)?);
    }
    let active: BTreeSet<AgentPubKey> = calls::active_among(agents, now)?.into_iter().collect();
    let mut members = Vec::new();
    for hash in all {
        let counts = match roll.agent(&hash)? {
            Some(agent) => {
                Some(&hash) == proposer
                    || (active.contains(&agent) && standing.full(&agent, now)?)
            }
            None => false,
        };
        if counts {
            members.push(hash);
        }
    }
    Ok(members)
}
