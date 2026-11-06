//! Service requests in a direct chat (ADR 0076, web's personal-service requests): the person
//! asking sends one (`send_service_request`), which also posts web's request line into the
//! chat; the service's creator accepts or declines, the person asking cancels
//! (`decide_service_request`), each posting web's status line. `service_requests` replays
//! every request here with `sp_service::replay`.

use hdk::prelude::*;
use messages_integrity::{
    EntryTypes, LinkTypes, Message, RequestStatus, ServiceRequest, UnitEntryTypes,
};
use sp_core::conversation::ChatKind;
use sp_service::{
    check_request, check_status, may_decide, replay, request_message, status_message,
};

use crate::entries::{all, path, Found};
use crate::people::load;
use crate::{guest, MESSAGES};

#[derive(Serialize, Deserialize, Debug)]
pub struct RequestInput {
    pub project: String,
    pub title: String,
    pub body: String,
    pub starts_at: Option<Timestamp>,
    pub ends_at: Option<Timestamp>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DecideInput {
    pub request: ActionHash,
    pub status: String,
    #[serde(default)]
    pub reason: String,
}

/// web's `ServiceRequestOut`, with the decision replayed.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RequestView {
    pub id: ActionHash,
    pub project: String,
    pub requester: AgentPubKey,
    pub title: String,
    pub body: String,
    /// `open`, `accepted`, `declined` or `cancelled`.
    pub status: String,
    pub decline_reason: Option<String>,
    pub starts_at: Option<Timestamp>,
    pub ends_at: Option<Timestamp>,
    pub created_at: Timestamp,
    /// When it was settled (web's `updated_at`), else when it was sent.
    pub updated_at: Timestamp,
}

/// web's `_request_href`.
fn href(project: &str, request: &ActionHash) -> String {
    format!(
        "/projects/{project}?tab=participation&request={}",
        ActionHashB64::from(request.clone())
    )
}

fn post_line(body: String, invite: Option<ActionHash>) -> ExternResult<()> {
    let message = Message {
        body,
        reply_to: None,
        invite,
        attachments: Vec::new(),
    };
    let hash = create_entry(EntryTypes::Message(message))?;
    create_link(path(MESSAGES)?, hash, LinkTypes::Messages, ())?;
    Ok(())
}

#[hdk_extern]
pub fn send_service_request(input: RequestInput) -> ExternResult<ActionHash> {
    let chat = load()?;
    chat.require_member()?;
    if chat.kind != ChatKind::Direct {
        return Err(guest(
            "Service requests go in a direct chat with the service's creator.",
        ));
    }
    let title = input.title.trim().to_string();
    let body = input.body.trim().to_string();
    let window = match (input.starts_at, input.ends_at) {
        (None, None) => None,
        (Some(s), Some(e)) => Some((s, e)),
        _ => return Err(guest("Add both a start and a finish time, or neither.")),
    };
    let micros = window.map(|(s, e)| (s.as_micros(), e.as_micros()));
    check_request(&input.project, &title, &body, micros).map_err(|r| guest(&r))?;
    let invite = chat.my_invite()?;
    let request = ServiceRequest {
        project: input.project.clone(),
        title: title.clone(),
        body: body.clone(),
        starts_at: input.starts_at,
        ends_at: input.ends_at,
        invite: invite.clone(),
    };
    let hash = create_entry(EntryTypes::ServiceRequest(request))?;
    create_link(path(MESSAGES)?, hash.clone(), LinkTypes::Requests, ())?;
    let when = window.map(|(s, e)| (s.to_string(), e.to_string()));
    let when = when.as_ref().map(|(s, e)| (s.as_str(), e.as_str()));
    post_line(
        request_message(&title, &body, when, &href(&input.project, &hash)),
        invite,
    )?;
    Ok(hash)
}

#[hdk_extern]
pub fn decide_service_request(input: DecideInput) -> ExternResult<RequestView> {
    let chat = load()?;
    chat.require_member()?;
    let reason = input.reason.trim().to_string();
    let reason = if input.status == "declined" {
        reason
    } else {
        String::new()
    };
    check_status(&input.status, &reason).map_err(|r| guest(&r))?;
    let ledger = Ledger::load()?;
    let Some(request) = ledger.requests.iter().find(|r| r.hash == input.request) else {
        return Err(guest("Service request not found"));
    };
    if !may_decide(&input.status, request.author == chat.me) {
        return Err(guest(if input.status == "cancelled" {
            "Only the requester can cancel this request"
        } else {
            "Only request reviewers can update service requests"
        }));
    }
    if ledger.view(request).status != "open" {
        return Err(guest(if input.status == "cancelled" {
            "Only a pending request can be cancelled"
        } else {
            "This request has already been settled."
        }));
    }
    let invite = chat.my_invite()?;
    let status = RequestStatus {
        request: input.request.clone(),
        status: input.status.clone(),
        reason: reason.clone(),
        invite: invite.clone(),
    };
    let hash = create_entry(EntryTypes::RequestStatus(status))?;
    create_link(path(MESSAGES)?, hash, LinkTypes::RequestStatuses, ())?;
    let line = status_message(
        &request.entry.title,
        &input.status,
        &reason,
        &href(&request.entry.project, &request.hash),
    );
    post_line(line, invite)?;
    Ok(Ledger::load()?.view(request))
}

/// Every request in this chat, newest first (web's `created_at desc`).
#[hdk_extern]
pub fn service_requests() -> ExternResult<Vec<RequestView>> {
    let ledger = Ledger::load()?;
    let mut out: Vec<RequestView> = ledger.requests.iter().map(|r| ledger.view(r)).collect();
    out.reverse();
    Ok(out)
}

struct Ledger {
    requests: Vec<Found<ServiceRequest>>,
    statuses: Vec<Found<RequestStatus>>,
}

impl Ledger {
    fn load() -> ExternResult<Self> {
        Ok(Self {
            requests: all(
                UnitEntryTypes::ServiceRequest,
                LinkTypes::Requests,
                MESSAGES,
            )?,
            statuses: all(
                UnitEntryTypes::RequestStatus,
                LinkTypes::RequestStatuses,
                MESSAGES,
            )?,
        })
    }

    fn view(&self, r: &Found<ServiceRequest>) -> RequestView {
        let mine: Vec<&Found<RequestStatus>> = self
            .statuses
            .iter()
            .filter(|s| s.entry.request == r.hash)
            .collect();
        let state = replay(mine.iter().map(|s| {
            (
                s.entry.status.as_str(),
                s.entry.reason.as_str(),
                s.author == r.author,
            )
        }));
        let settled = mine.iter().find(|s| {
            s.entry.status == state.id() && may_decide(&s.entry.status, s.author == r.author)
        });
        let decline_reason = match &state {
            sp_service::State::Declined { reason } if !reason.is_empty() => Some(reason.clone()),
            _ => None,
        };
        RequestView {
            id: r.hash.clone(),
            project: r.entry.project.clone(),
            requester: r.author.clone(),
            title: r.entry.title.clone(),
            body: r.entry.body.clone(),
            status: state.id().to_string(),
            decline_reason,
            starts_at: r.entry.starts_at,
            ends_at: r.entry.ends_at,
            created_at: r.at,
            updated_at: settled.map_or(r.at, |s| s.at),
        }
    }
}
