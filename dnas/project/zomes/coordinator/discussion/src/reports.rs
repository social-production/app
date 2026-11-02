//! Reports (ADR 0071): web's ⋮ Report on a space (project, event, help request, thread, post).
//! The state is replayed from the votes (`sp_moderation::replay`); only voters with Full
//! standing at the time of their vote count, as for proposals (ADR 0033).

use std::collections::{BTreeMap, BTreeSet};

use discussion_integrity::{EntryTypes, LinkTypes, Report};
use hdk::prelude::*;
use sp_moderation::{
    audience, check_report, popularity_score, replay, Ballot, Reason, Resolution, Thresholds, NO,
    YES,
};

use crate::report_links::{self, Seen};
use crate::{anchor, guest, reads, scope};

#[derive(Serialize, Deserialize, Debug)]
pub struct ReportInput {
    pub reason: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReportVoteInput {
    pub report: ActionHash,
    pub yes: bool,
}

/// web's `ContentReportSummary` for one report, with the bars it is measured against.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReportView {
    pub id: ActionHash,
    pub reporter: AgentPubKey,
    pub reason: String,
    pub description: String,
    pub created_at: Timestamp,
    /// web's resolution id: `open`, `under_review`, `hidden`, `removed` or `dismissed`.
    pub resolution: String,
    pub yes: u32,
    pub no: u32,
    /// The caller's latest vote, counted or not.
    pub mine: Option<bool>,
    pub audience: u32,
    pub delete_quorum: u32,
    pub hide_quorum: u32,
    /// Whole percentages.
    pub delete_share: u32,
    pub hide_share: u32,
}

/// What the space's reports add up to: web's moderation state and the report to show.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModerationView {
    /// `visible`, `under_review`, `hidden` or `removed`.
    pub state: String,
    /// The removing report, else the earliest still open; `None` when nothing is open.
    pub report: Option<ReportView>,
}

/// The item's audience and popularity, as web's `electorate_size_for_target` and
/// `load_target_engagement` (members, commenters and signallers; ADR 0071 "Audience").
struct Engagement {
    author: AgentPubKey,
    created_at: i64,
    members: u32,
    commenters: u32,
    signallers: u32,
    score: u32,
}

#[derive(Deserialize, Debug)]
struct SignalCounts {
    demand: u32,
    opposition: u32,
}

fn engagement() -> ExternResult<Engagement> {
    let space = anchor()?;
    let charter = must_get_action(space.clone())?;
    let members: Vec<ActionHash> = scope::call_project("member_snapshot")?;
    let signals: SignalCounts = scope::call_project("signal_counts")?;
    let author = charter.action().author().clone();
    let commenters: BTreeSet<AgentPubKey> = report_links::on(&space, LinkTypes::ChatMessages)?
        .into_iter()
        .map(|s| s.author)
        .filter(|a| *a != author)
        .collect();
    let signallers = signals.demand + signals.opposition;
    let commenters = count(commenters.len());
    Ok(Engagement {
        author,
        created_at: charter.action().timestamp().as_micros(),
        members: count(members.len()),
        commenters,
        signallers,
        score: popularity_score(signallers, commenters),
    })
}

fn count(n: usize) -> u32 {
    u32::try_from(n).unwrap_or(u32::MAX)
}

fn decode(hash: &ActionHash) -> ExternResult<Option<(Record, Report)>> {
    let Some(record) = reads::record(hash)? else {
        return Ok(None);
    };
    let report = record.entry().to_app_option::<Report>().ok().flatten();
    Ok(report.map(|r| (record, r)))
}

/// The space's reports, oldest first.
fn reports() -> ExternResult<Vec<(Record, Report)>> {
    let mut out = Vec::new();
    for seen in report_links::on(&anchor()?, LinkTypes::Reports)? {
        if let Some(hash) = seen.target.into_action_hash() {
            if let Some(found) = decode(&hash)? {
                out.push(found);
            }
        }
    }
    out.sort_by_key(|(r, _)| r.action().timestamp());
    out.dedup_by(|a, b| a.0.action_address() == b.0.action_address());
    Ok(out)
}

#[derive(Serialize, Debug)]
struct StandingAt {
    agent: AgentPubKey,
    at: Timestamp,
}

fn full_before(agent: &AgentPubKey, at: Timestamp) -> ExternResult<bool> {
    let query = StandingAt {
        agent: agent.clone(),
        at,
    };
    match call(
        CallTargetCell::OtherRole("directory".into()),
        ZomeName::from("identity"),
        "full_standing_before".into(),
        None,
        query,
    )? {
        ZomeCallResponse::Ok(io) => io.decode().map_err(|e| wasm_error!(e)),
        other => Err(guest(&format!("identity/full_standing_before: {other:?}"))),
    }
}

fn view(record: &Record, report: &Report, e: &Engagement) -> ExternResult<ReportView> {
    let reason = Reason::parse(&report.reason).unwrap_or(Reason::Spam);
    let me = agent_info()?.agent_initial_pubkey;
    let votes: Vec<Seen> = report_links::on(record.action_address(), LinkTypes::ReportVotes)?;
    let mut full: BTreeMap<(AgentPubKey, Timestamp), bool> = BTreeMap::new();
    let mut ballots = Vec::new();
    let mut mine = None;
    for v in votes.iter().filter(|v| v.author != e.author) {
        let yes = v.tag.as_slice() == [YES];
        if v.author == me {
            mine = Some(yes);
        }
        let key = (v.author.clone(), v.at);
        let counted = match full.get(&key) {
            Some(known) => *known,
            None => *full.entry(key).or_insert(full_before(&v.author, v.at)?),
        };
        if counted {
            ballots.push(Ballot {
                at: v.at.as_micros(),
                voter: v.author.clone(),
                yes,
            });
        }
    }
    let voters: BTreeSet<&AgentPubKey> = ballots.iter().map(|b| &b.voter).collect();
    let audience = audience(e.members, e.commenters, e.signallers, count(voters.len()));
    let out = replay(reason, audience, e.score, e.created_at, &ballots);
    let now = sys_time()?.as_micros();
    let t = Thresholds::at(
        reason,
        audience,
        sp_moderation::age_days(e.created_at, now),
        e.score,
    );
    Ok(ReportView {
        id: record.action_address().clone(),
        reporter: record.action().author().clone(),
        reason: report.reason.clone(),
        description: report.description.clone(),
        created_at: record.action().timestamp(),
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

/// web's `moderation_state` and active report: a removing report wins; otherwise the
/// earliest open one (web keeps one active report per item).
pub fn status() -> ExternResult<ModerationView> {
    let found = reports()?;
    if found.is_empty() {
        return Ok(ModerationView {
            state: "visible".into(),
            report: None,
        });
    }
    let e = engagement()?;
    let mut active: Option<ReportView> = None;
    for (record, report) in &found {
        let v = view(record, report, &e)?;
        if v.resolution == Resolution::Removed.id() {
            return Ok(ModerationView {
                state: "removed".into(),
                report: Some(v),
            });
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
    Ok(ModerationView {
        state: state.into(),
        report: active,
    })
}

/// web's `submit_report`: one active report per item; the reporter's yes comes with it.
pub fn submit(input: &ReportInput) -> ExternResult<ModerationView> {
    check_report(&input.reason, &input.description).map_err(|r| guest(&r))?;
    let space = anchor()?;
    let me = agent_info()?.agent_initial_pubkey;
    if must_get_action(space.clone())?.action().author() == &me {
        return Err(guest("You can't report yourself"));
    }
    if status()?.report.is_some() {
        return Err(guest(
            "This is already being reviewed. Vote on that report instead.",
        ));
    }
    let report = Report {
        target: space.clone(),
        reason: input.reason.clone(),
        description: input.description.trim().to_string(),
    };
    let hash = create_entry(EntryTypes::Report(report))?;
    create_link(space, hash.clone(), LinkTypes::Reports, ())?;
    create_link(hash, me, LinkTypes::ReportVotes, LinkTag::new(vec![YES]))?;
    scope::touch()?;
    status()
}

/// web's `vote_report`: yes or no on the item's open report; a later vote replaces yours.
pub fn vote(input: &ReportVoteInput) -> ExternResult<ModerationView> {
    let current = status()?;
    let Some(open) = current.report.filter(|r| r.id == input.report) else {
        return Err(guest("This report is already resolved."));
    };
    if open.resolution == Resolution::Removed.id() {
        return Err(guest("This report is already resolved."));
    }
    let me = agent_info()?.agent_initial_pubkey;
    if !full_before(&me, sys_time()?)? {
        return Err(guest("Only full members vote on reports."));
    }
    let tag = if input.yes { YES } else { NO };
    create_link(
        input.report.clone(),
        me,
        LinkTypes::ReportVotes,
        LinkTag::new(vec![tag]),
    )?;
    scope::touch()?;
    status()
}
