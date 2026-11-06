//! A personal service's weekly hours and bookings (web-backend
//! `projects_service_availability.py` and the booking in `update_service_request_status`;
//! ADR 0077). Only the service's creator writes either.

use hdk::prelude::*;
use project_integrity::{
    Activity, ActivityRole, AvailabilityRule, Booking, EntryTypes, LinkTypes, UnitEntryTypes,
};
use sp_activity::availability::{check_rule, PROVIDER};
use sp_activity::{check_activity, roles_open, TITLE_MAX};

use crate::calls;
use crate::reads;
use crate::write::{guest, spec};

#[derive(Serialize, Deserialize, Debug)]
pub struct RuleInput {
    /// 0 = Monday … 6 = Sunday (web's `weekday`).
    pub weekday: u8,
    /// Minutes after midnight in `zone`.
    pub start: u16,
    pub end: u16,
    pub zone: String,
    pub note: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RuleView {
    pub id: ActionHash,
    pub weekday: u8,
    pub start: u16,
    pub end: u16,
    pub zone: String,
    pub note: String,
}

/// web's `_ensure_personal_service_creator`.
fn ensure_creator() -> ExternResult<calls::Glance> {
    let project = calls::glance()?;
    if project.charter.as_ref().map(|c| c.mode.as_str()) != Some("personal-service") {
        return Err(guest(
            "Only personal-service projects support availability rules",
        ));
    }
    if project.creator != agent_info()?.agent_initial_pubkey {
        return Err(guest("Only the service creator can edit availability"));
    }
    Ok(project)
}

pub fn add_rule(input: RuleInput) -> ExternResult<ActionHash> {
    ensure_creator()?;
    let zone = input.zone.trim();
    let zone = if zone.is_empty() { "UTC" } else { zone };
    let note = input.note.trim();
    check_rule(input.weekday, input.start, input.end, zone, note).map_err(guest)?;
    let space = space_scope::current()?;
    let hash = create_entry(EntryTypes::AvailabilityRule(AvailabilityRule {
        space: space.clone(),
        weekday: input.weekday,
        start: input.start,
        end: input.end,
        zone: zone.to_string(),
        note: note.to_string(),
    }))?;
    create_link(space, hash.clone(), LinkTypes::Availability, ())?;
    Ok(hash)
}

/// web's `delete_availability_rule`: the rule's link goes, so the hours stop showing.
pub fn remove_rule(rule: ActionHash) -> ExternResult<()> {
    ensure_creator()?;
    let query = LinkQuery::try_new(reads::anchor()?, LinkTypes::Availability)?;
    let mine = agent_info()?.agent_initial_pubkey;
    let links = get_links(query, GetStrategy::Local)?;
    let mut found = false;
    for link in links {
        if link.author == mine && link.target.clone().into_action_hash().as_ref() == Some(&rule) {
            delete_link(link.create_link_hash, GetOptions::default())?;
            found = true;
        }
    }
    if found {
        Ok(())
    } else {
        Err(guest("Availability rule not found"))
    }
}

/// The service's weekly hours, by weekday then start (web's `list_availability_rules`).
pub fn list_rules() -> ExternResult<Vec<RuleView>> {
    let creator = calls::glance()?.creator;
    let query = LinkQuery::try_new(reads::anchor()?, LinkTypes::Availability)?;
    let mut out = Vec::new();
    for link in get_links(query, GetStrategy::default())? {
        let Some(hash) = link.target.into_action_hash() else {
            continue;
        };
        let Some(r) = reads::found::<AvailabilityRule>(hash)? else {
            continue;
        };
        if r.author == creator {
            out.push(RuleView {
                id: r.hash,
                weekday: r.entry.weekday,
                start: r.entry.start,
                end: r.entry.end,
                zone: r.entry.zone,
                note: r.entry.note,
            });
        }
    }
    out.sort_by_key(|r| (r.weekday, r.start));
    out.dedup_by(|a, b| a.id == b.id);
    Ok(out)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BookInput {
    /// The accepted request, in the direct chat (ADR 0076).
    pub request: ActionHash,
    pub requester: AgentPubKey,
    pub title: String,
    pub starts_at: Timestamp,
    pub ends_at: Timestamp,
}

/// web: an accepted timed request becomes a booking, an activity whose one Provider role the
/// creator holds. Booking the same request again returns the first booking.
pub fn book(input: BookInput) -> ExternResult<ActionHash> {
    let project = ensure_creator()?;
    let membership = calls::my_membership()?
        .ok_or_else(|| guest("Join the project to take part in activities."))?;
    let mine = reads::linked::<Activity>(
        reads::anchor()?,
        LinkTypes::Activities,
        UnitEntryTypes::Activity,
    )?;
    let earlier = mine.into_iter().find(|a| {
        a.entry
            .booking
            .as_ref()
            .is_some_and(|b| b.request == input.request)
    });
    if let Some(earlier) = earlier {
        return Ok(earlier.hash);
    }
    let activity = booking_of(input, &project, membership.clone());
    check_activity(&spec(&activity)).map_err(guest)?;
    let ends_at = activity.ends_at.as_micros();
    let hash = create_entry(EntryTypes::Activity(activity))?;
    create_link(reads::anchor()?, hash.clone(), LinkTypes::Activities, ())?;
    // The provider holds the booking's one role, so history counts them as its participant.
    if roles_open(ends_at, sys_time()?.as_micros()) {
        let mut tag = vec![0];
        tag.extend_from_slice(membership.get_raw_39());
        create_link(
            hash.clone(),
            agent_info()?.agent_initial_pubkey,
            LinkTypes::Assignments,
            LinkTag::new(tag),
        )?;
    }
    Ok(hash)
}

/// web's booking: the request's title and time at the service's place, one Provider role.
fn booking_of(input: BookInput, project: &calls::Glance, membership: ActionHash) -> Activity {
    let title: String = input.title.trim().chars().take(TITLE_MAX).collect();
    let place = project
        .charter
        .as_ref()
        .map(|c| c.location_label.trim().to_string())
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| "Online".into());
    // The service's own place, when it has coordinates and is the label used (ADR 0078).
    let location = project
        .charter
        .as_ref()
        .and_then(|c| c.place.clone())
        .filter(|p| !p.online && p.label == place);
    Activity {
        title: if title.is_empty() {
            "Booking".into()
        } else {
            title
        },
        starts_at: input.starts_at,
        ends_at: input.ends_at,
        online: false,
        place,
        location,
        meeting_url: None,
        note: "Booked through a service request.".into(),
        plan_stage: None,
        roles: vec![ActivityRole {
            label: PROVIDER.into(),
            required: 1,
            maximum: Some(1),
            description: String::new(),
        }],
        membership,
        organizer: None,
        booking: Some(Booking {
            request: input.request,
            requester: input.requester,
        }),
    }
}
