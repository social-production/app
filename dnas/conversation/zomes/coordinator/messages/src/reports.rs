//! Reports on direct and group chat messages (ADR 0075, web's `message` target). A member
//! reports someone else's message with their yes vote; the other members vote. Every reader
//! replays the votes with web's numbers for chat messages (`sp_moderation::replay_chat`):
//! the audience is the chat's members other than the author, popularity is 0, and in a 1:1
//! chat the report's own yes removes the message at once.

use hdk::prelude::*;
use messages_integrity::{EntryTypes, LinkTypes, Report, ReportVote, UnitEntryTypes};
use sp_core::chat::PAGE_MAX;
use sp_moderation::{age_days, check_report, replay_chat, Ballot, Reason, Resolution, Thresholds};

use crate::entries::{all, path, Found};
use crate::people::{load, Chat};
use crate::{guest, reads, MESSAGES};

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageReportInput {
    pub message: ActionHash,
    pub reason: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReportVoteInput {
    pub report: ActionHash,
    pub yes: bool,
}

/// web's `ContentReportSummary` (the project DNA's `ReportView` shape).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReportView {
    pub id: ActionHash,
    pub reporter: AgentPubKey,
    pub reason: String,
    pub description: String,
    pub created_at: Timestamp,
    pub resolution: String,
    pub yes: u32,
    pub no: u32,
    pub mine: Option<bool>,
    pub audience: u32,
    pub delete_quorum: u32,
    pub hide_quorum: u32,
    pub delete_share: u32,
    pub hide_share: u32,
}

/// `visible`, `under_review`, `hidden` or `removed`, with the report to show.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModerationView {
    pub state: String,
    pub report: Option<ReportView>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageModeration {
    pub id: ActionHash,
    pub moderation: ModerationView,
}

/// The chat's reports and votes, read once per call.
struct Ledger {
    chat: Chat,
    reports: Vec<Found<Report>>,
    votes: Vec<Found<ReportVote>>,
}

impl Ledger {
    fn load() -> ExternResult<Self> {
        Ok(Self {
            chat: load()?,
            reports: all(UnitEntryTypes::Report, LinkTypes::Reports, MESSAGES)?,
            votes: all(UnitEntryTypes::ReportVote, LinkTypes::ReportVotes, MESSAGES)?,
        })
    }

    /// The members other than the message's author (web's `max(1, members - 1)`).
    fn audience(&self) -> u32 {
        let n = self.chat.roster.members.len().saturating_sub(1).max(1);
        u32::try_from(n).unwrap_or(u32::MAX)
    }

    fn view(&self, report: &Found<Report>, message: &Record) -> ExternResult<ReportView> {
        let author = message.action().author();
        let created = message.action().timestamp().as_micros();
        let reason = Reason::parse(&report.entry.reason).unwrap_or(Reason::Spam);
        let mut ballots = Vec::new();
        let mut mine = None;
        for v in self.votes.iter().filter(|v| v.entry.report == report.hash) {
            if &v.author == author || !self.chat.roster.in_at(&v.author, v.at) {
                continue;
            }
            if v.author == self.chat.me {
                mine = Some(v.entry.yes);
            }
            ballots.push(Ballot {
                at: v.at.as_micros(),
                voter: v.author.clone(),
                yes: v.entry.yes,
            });
        }
        let out = replay_chat(reason, self.audience(), created, &ballots);
        let now = sys_time()?.as_micros();
        let t = Thresholds::chat(reason, self.audience(), age_days(created, now));
        Ok(ReportView {
            id: report.hash.clone(),
            reporter: report.author.clone(),
            reason: report.entry.reason.clone(),
            description: report.entry.description.clone(),
            created_at: report.at,
            resolution: out.resolution.id().to_string(),
            yes: out.yes,
            no: out.no,
            mine,
            audience: t.audience,
            delete_quorum: t.delete_quorum,
            hide_quorum: t.hide_quorum,
            delete_share: t.delete_share,
            hide_share: t.hide_share,
        })
    }

    /// What the message's reports add up to: a removing one wins, else the earliest open.
    fn status(&self, message: &Record) -> ExternResult<ModerationView> {
        let id = message.action_address();
        let mut active = None;
        for report in self.reports.iter().filter(|r| &r.entry.message == id) {
            if !self.chat.roster.in_at(&report.author, report.at) {
                continue;
            }
            let v = self.view(report, message)?;
            if v.resolution == Resolution::Removed.id() {
                return Ok(moderation("removed", Some(v)));
            }
            if active.is_none() && v.resolution != Resolution::Dismissed.id() {
                active = Some(v);
            }
        }
        let state = match active.as_ref().map(|v| v.resolution.as_str()) {
            Some("hidden") => "hidden",
            Some(_) => "under_review",
            None => "visible",
        };
        Ok(moderation(state, active))
    }

    fn reported(&self, id: &ActionHash) -> bool {
        self.reports.iter().any(|r| &r.entry.message == id)
    }
}

fn moderation(state: &str, report: Option<ReportView>) -> ModerationView {
    ModerationView {
        state: state.into(),
        report,
    }
}

fn message(hash: &ActionHash) -> ExternResult<Record> {
    match reads::record(hash)? {
        Some(r) if reads::decode(&r)?.is_some() => Ok(r),
        _ => Err(guest("This message isn't here yet. Try again in a moment.")),
    }
}

/// The reported ones among `ids`, each with its state (the rest are visible).
#[hdk_extern]
pub fn message_moderation(ids: Vec<ActionHash>) -> ExternResult<Vec<MessageModeration>> {
    if ids.len() > PAGE_MAX {
        return Err(guest("Too many messages at once."));
    }
    let ledger = Ledger::load()?;
    let mut out = Vec::new();
    for id in ids.iter().filter(|id| ledger.reported(id)) {
        let Some(record) = reads::record(id)? else {
            continue;
        };
        let moderation = ledger.status(&record)?;
        if moderation.report.is_some() || moderation.state != "visible" {
            out.push(MessageModeration {
                id: id.clone(),
                moderation,
            });
        }
    }
    Ok(out)
}

/// web's `submit_report` on a message: one active report per message, never your own; your
/// yes vote comes with it.
#[hdk_extern]
pub fn report_message(input: MessageReportInput) -> ExternResult<ModerationView> {
    check_report(&input.reason, &input.description).map_err(|r| guest(&r))?;
    let ledger = Ledger::load()?;
    ledger.chat.require_member()?;
    let invite = ledger.chat.my_invite()?;
    let record = message(&input.message)?;
    if record.action().author() == &ledger.chat.me {
        return Err(guest("You can't report yourself"));
    }
    if ledger.status(&record)?.report.is_some() {
        return Err(guest(
            "This is already being reviewed. Vote on that report instead.",
        ));
    }
    let report = Report {
        message: input.message.clone(),
        reason: input.reason,
        description: input.description.trim().to_string(),
        invite: invite.clone(),
    };
    let hash = create_entry(EntryTypes::Report(report))?;
    create_link(path(MESSAGES)?, hash.clone(), LinkTypes::Reports, ())?;
    cast(hash, true, invite)?;
    Ledger::load()?.status(&record)
}

/// web's `vote_report`: yes or no on a message's open report; a later vote replaces yours.
#[hdk_extern]
pub fn vote_report(input: ReportVoteInput) -> ExternResult<ModerationView> {
    let ledger = Ledger::load()?;
    ledger.chat.require_member()?;
    let Some(report) = ledger.reports.iter().find(|r| r.hash == input.report) else {
        return Err(guest("This report isn't here yet. Try again in a moment."));
    };
    let record = message(&report.entry.message)?;
    if record.action().author() == &ledger.chat.me {
        return Err(guest("Not eligible to vote on this report"));
    }
    let open = ledger
        .status(&record)?
        .report
        .filter(|r| r.id == input.report);
    if open.is_none_or(|r| r.resolution == Resolution::Removed.id()) {
        return Err(guest("This report is already resolved."));
    }
    cast(input.report, input.yes, ledger.chat.my_invite()?)?;
    Ledger::load()?.status(&record)
}

fn cast(report: ActionHash, yes: bool, invite: Option<ActionHash>) -> ExternResult<()> {
    let vote = ReportVote {
        report,
        yes,
        invite,
    };
    let hash = create_entry(EntryTypes::ReportVote(vote))?;
    create_link(path(MESSAGES)?, hash, LinkTypes::ReportVotes, ())?;
    Ok(())
}
