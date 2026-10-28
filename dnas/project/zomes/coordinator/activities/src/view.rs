//! What web's activity cards show (web-backend `hydrate.py` activities): roles with the people
//! who hold them (`sp_activity::fill`), the standing suggestion per role, ratings.

use std::collections::BTreeMap;

use hdk::prelude::*;
use project_integrity::{
    Activity, ActivityRating, LinkTypes, RoleSuggestion, SuggestionDecline, UnitEntryTypes,
};
use sp_activity::{fill, roles_open, Taken};

use crate::calls;
use crate::history::{self, CompletionView};
use crate::reads::{self, Found};

#[derive(Serialize, Deserialize, Debug)]
pub struct SuggestionView {
    pub id: ActionHash,
    pub person: AgentPubKey,
    pub by: AgentPubKey,
    pub at: Timestamp,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoleView {
    pub label: String,
    pub required: u32,
    pub maximum: Option<u32>,
    /// A help request's role description (ADR 0067).
    pub description: String,
    /// Who holds it, in the order they took it, and when each took it.
    pub people: Vec<AgentPubKey>,
    pub taken_at: Vec<Timestamp>,
    pub suggestion: Option<SuggestionView>,
    /// Suggestions their person turned down (`id` and `at` are the decline's), which web
    /// tells the suggester about.
    pub declined: Vec<SuggestionView>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RatingView {
    pub author: AgentPubKey,
    pub stars: u8,
    pub comment: String,
    pub at: Timestamp,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActivityView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub created_at: Timestamp,
    pub title: String,
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
    pub online: bool,
    pub place: String,
    pub meeting_url: Option<String>,
    pub note: String,
    pub plan_stage: Option<String>,
    pub roles: Vec<RoleView>,
    /// The role the caller holds, if any.
    pub my_role: Option<u8>,
    pub roles_open: bool,
    pub ratings: Vec<RatingView>,
    /// The caller took a role here, so may rate once it has ended.
    pub can_rate: bool,
    /// Participants' check-ins (meaningful once it has ended).
    pub completion: CompletionView,
}

/// Every activity, soonest first.
pub fn list() -> ExternResult<Vec<ActivityView>> {
    let all = reads::linked::<Activity>(
        reads::anchor()?,
        LinkTypes::Activities,
        UnitEntryTypes::Activity,
    )?;
    let mut here = Vec::new();
    for a in all {
        if reads::in_space(&a.entry.membership)? {
            here.push(a);
        }
    }
    let all = here;
    let members = calls::glance()?.members;
    let me = agent_info()?.agent_initial_pubkey;
    let now = sys_time()?.as_micros();
    let mut out = all
        .into_iter()
        .map(|a| one(a, &members, &me, now))
        .collect::<ExternResult<Vec<_>>>()?;
    out.sort_by_key(|a| (a.starts_at, a.created_at));
    Ok(out)
}

fn one(
    a: Found<Activity>,
    members: &[AgentPubKey],
    me: &AgentPubKey,
    now: i64,
) -> ExternResult<ActivityView> {
    let open = roles_open(a.entry.ends_at.as_micros(), now);
    let holders = holders_among(&a.hash, &a.entry, |agent| !open || members.contains(agent))?;
    let my_role = holders
        .iter()
        .position(|p| p.contains(me))
        .and_then(|i| u8::try_from(i).ok());
    let everyone: Vec<AgentPubKey> = holders.iter().flatten().cloned().collect();
    let completion = if open {
        CompletionView::default()
    } else {
        history::completion(&a.hash, &everyone, me)?
    };
    let (mut suggestions, mut declined) = suggestions(&a.hash)?;
    let takes = reads::takes(&a.hash)?;
    let taken_at = |role: usize, agent: &AgentPubKey| {
        let mine = takes
            .iter()
            .filter(|t| usize::from(t.role) == role && t.agent == *agent);
        mine.map(|t| t.at).min().unwrap_or(a.at)
    };
    let roles = a
        .entry
        .roles
        .iter()
        .zip(holders)
        .enumerate()
        .map(|(i, (r, people))| RoleView {
            label: r.label.clone(),
            description: r.description.clone(),
            required: r.required,
            maximum: r.maximum,
            taken_at: people.iter().map(|p| taken_at(i, p)).collect(),
            people,
            suggestion: u8::try_from(i).ok().and_then(|i| suggestions.remove(&i)),
            declined: u8::try_from(i)
                .ok()
                .and_then(|i| declined.remove(&i))
                .unwrap_or_default(),
        })
        .collect();
    Ok(ActivityView {
        can_rate: my_take(&a.hash, me)?.is_some(),
        ratings: ratings(&a.hash)?,
        completion,
        id: a.hash,
        author: a.author,
        created_at: a.at,
        title: a.entry.title,
        starts_at: a.entry.starts_at,
        ends_at: a.entry.ends_at,
        online: a.entry.online,
        place: a.entry.place,
        meeting_url: a.entry.meeting_url,
        note: a.entry.note,
        plan_stage: a.entry.plan_stage,
        roles,
        my_role,
        roles_open: open,
    })
}

/// Who holds each role now; people who left the project count only once it has ended.
pub fn holders(hash: &ActionHash, activity: &Activity) -> ExternResult<Vec<Vec<AgentPubKey>>> {
    let open = roles_open(activity.ends_at.as_micros(), sys_time()?.as_micros());
    let members = calls::glance()?.members;
    holders_among(hash, activity, |agent| !open || members.contains(agent))
}

fn holders_among(
    hash: &ActionHash,
    activity: &Activity,
    counts: impl Fn(&AgentPubKey) -> bool,
) -> ExternResult<Vec<Vec<AgentPubKey>>> {
    let taken = reads::takes(hash)?
        .into_iter()
        .map(|t| Taken {
            member: counts(&t.agent),
            role: usize::from(t.role),
            at: t.at.as_micros(),
            hash: t.link.get_raw_39().to_vec(),
            agent: t.agent.get_raw_39().to_vec(),
        })
        .collect();
    let maxima: Vec<Option<u32>> = activity.roles.iter().map(|r| r.maximum).collect();
    Ok(fill(taken, &maxima)
        .into_iter()
        .map(|people| people.into_iter().map(AgentPubKey::from_raw_39).collect())
        .collect())
}

/// The caller's live `Assignments` link on `activity`, which a rating cites.
pub fn my_take(activity: &ActionHash, me: &AgentPubKey) -> ExternResult<Option<ActionHash>> {
    Ok(reads::takes(activity)?
        .into_iter()
        .find(|t| t.agent == *me)
        .map(|t| t.link))
}

type PerRole<T> = BTreeMap<u8, T>;

/// The latest suggestion per role that its person hasn't declined, and every decline per role.
fn suggestions(
    activity: &ActionHash,
) -> ExternResult<(PerRole<SuggestionView>, PerRole<Vec<SuggestionView>>)> {
    let made: Vec<Found<RoleSuggestion>> = reads::linked::<RoleSuggestion>(
        activity.clone(),
        LinkTypes::ActivityToSuggestions,
        UnitEntryTypes::RoleSuggestion,
    )?
    .into_iter()
    .filter(|s| s.entry.activity == *activity)
    .collect();
    let declines = reads::linked::<SuggestionDecline>(
        activity.clone(),
        LinkTypes::ActivityToSuggestions,
        UnitEntryTypes::SuggestionDecline,
    )?;
    let mut declined: PerRole<Vec<SuggestionView>> = BTreeMap::new();
    for d in &declines {
        let Some(s) = made
            .iter()
            .find(|s| s.hash == d.entry.suggestion && s.entry.person == d.author)
        else {
            continue;
        };
        declined
            .entry(s.entry.role)
            .or_default()
            .push(SuggestionView {
                id: d.hash.clone(),
                person: d.author.clone(),
                by: s.author.clone(),
                at: d.at,
            });
    }
    let mut latest: PerRole<&Found<RoleSuggestion>> = BTreeMap::new();
    for s in &made {
        latest.insert(s.entry.role, s);
    }
    let standing = latest
        .into_iter()
        .filter(|(_, s)| {
            !declines
                .iter()
                .any(|d| d.entry.suggestion == s.hash && d.author == s.entry.person)
        })
        .map(|(role, s)| {
            (
                role,
                SuggestionView {
                    id: s.hash.clone(),
                    person: s.entry.person.clone(),
                    by: s.author.clone(),
                    at: s.at,
                },
            )
        })
        .collect();
    Ok((standing, declined))
}

/// Each participant's latest rating, unless they removed it.
fn ratings(activity: &ActionHash) -> ExternResult<Vec<RatingView>> {
    let all = reads::linked::<ActivityRating>(
        activity.clone(),
        LinkTypes::ActivityToRatings,
        UnitEntryTypes::ActivityRating,
    )?;
    let mut latest: BTreeMap<AgentPubKey, Found<ActivityRating>> = BTreeMap::new();
    for r in all.into_iter().filter(|r| r.entry.activity == *activity) {
        latest.insert(r.author.clone(), r);
    }
    Ok(latest
        .into_values()
        .filter_map(|r| {
            Some(RatingView {
                stars: r.entry.stars?,
                comment: r.entry.comment,
                author: r.author,
                at: r.at,
            })
        })
        .collect())
}
