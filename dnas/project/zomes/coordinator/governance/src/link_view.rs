//! What the Links tab shows (web's `build_links_frame`, ADR 0042). A cell sees only its own
//! halves, so the UI reads the other projects' halves (`link_sides` there) and hands them
//! back here to be paired.

use std::collections::BTreeSet;

use hdk::prelude::*;
use project_integrity::{LinkRequest, ProposalKind};
use sp_governance::links::{pair, request_status, Half, SideStatus};
use sp_governance::Verdict;

use crate::apply::Standing;
use crate::roll::Roll;
use crate::views::{tally_view, TallyView};
use crate::{calls, decide, reads};

/// One project's half of a link request, with its votes.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HalfView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub created_at: Timestamp,
    pub request: LinkRequest,
    pub tally: TallyView,
    pub status: SideStatus,
}

impl HalfView {
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.status == SideStatus::Open
    }
}

/// A project's halves, as `link_sides` reports them to the other projects.
#[derive(Serialize, Deserialize, Debug)]
pub struct LinkSides {
    pub network_seed: String,
    pub am_member: bool,
    pub halves: Vec<HalfView>,
}

/// A request with both halves; `theirs` is missing until the other project has its half.
#[derive(Serialize, Deserialize, Debug)]
pub struct RequestView {
    pub mine: HalfView,
    pub theirs: Option<HalfView>,
    pub status: SideStatus,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LinkView {
    pub create: RequestView,
    /// The open sever vote, if any.
    pub sever: Option<RequestView>,
}

/// web's links frame: active and historical links, pending requests and past votes.
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LinksView {
    pub active: Vec<LinkView>,
    pub historical: Vec<LinkView>,
    pub pending: Vec<RequestView>,
    /// Rejected link votes and decided sever votes, newest first.
    pub past: Vec<RequestView>,
}

/// This cell's halves, oldest first. A half passes when the full recount does (ADR 0033);
/// one whose outcome stands but whose recount fails can't pass any more.
pub fn halves() -> ExternResult<Vec<HalfView>> {
    let me = agent_info()?.agent_initial_pubkey;
    let roll = Roll::load()?;
    let passed: BTreeSet<ActionHash> = decide::passed(&mut Standing::default(), &roll)?
        .into_iter()
        .map(|d| d.proposal.hash)
        .collect();
    let mut out: Vec<HalfView> = Vec::new();
    for p in reads::proposals()? {
        let ProposalKind::Link(request) = p.entry.kind.clone() else {
            continue;
        };
        // A request mirrored twice counts once: the earliest.
        if request.source.is_some() && out.iter().any(|h| h.request.source == request.source) {
            continue;
        }
        let tally = tally_view(&p, &me, &roll)?;
        let status = if passed.contains(&p.hash) {
            SideStatus::Approved
        } else if tally.status == Verdict::Open {
            SideStatus::Open
        } else {
            SideStatus::Rejected
        };
        out.push(HalfView {
            id: p.hash,
            author: p.author,
            created_at: p.at,
            request,
            tally,
            status,
        });
    }
    Ok(out)
}

pub fn sides() -> ExternResult<LinkSides> {
    Ok(LinkSides {
        network_seed: dna_info()?.modifiers.network_seed,
        am_member: calls::my_membership()?.is_some(),
        halves: halves()?,
    })
}

fn half(h: &HalfView) -> Half<ActionHash> {
    Half {
        id: h.id.clone(),
        source: h.request.source.clone(),
        sever: h.request.sever.clone(),
        relationship: h.request.relationship.clone(),
        summary: h.request.summary.clone(),
        status: h.status,
    }
}

/// Pair this cell's halves with `others` (each project's `link_sides`, any order).
pub fn view(others: &[LinkSides]) -> ExternResult<LinksView> {
    let seed = dna_info()?.modifiers.network_seed;
    let mine = halves()?;
    let mut theirs: Vec<Option<HalfView>> = vec![None; mine.len()];
    for o in others {
        let here: Vec<usize> = (0..mine.len())
            .filter(|&i| mine[i].request.other.network_seed == o.network_seed)
            .collect();
        let back: Vec<&HalfView> = o
            .halves
            .iter()
            .filter(|h| h.request.other.network_seed == seed)
            .collect();
        let found = pair(
            &here.iter().map(|&i| half(&mine[i])).collect::<Vec<_>>(),
            &back.iter().map(|h| half(h)).collect::<Vec<_>>(),
        );
        for (&i, f) in here.iter().zip(found) {
            theirs[i] = f.map(|j| back[j].clone());
        }
    }
    let requests = mine.into_iter().zip(theirs).map(|(m, t)| request(m, t));
    Ok(group(requests.collect()))
}

fn request(mine: HalfView, theirs: Option<HalfView>) -> RequestView {
    let status = request_status(mine.status, theirs.as_ref().map(|t| t.status));
    RequestView {
        mine,
        theirs,
        status,
    }
}

/// web's sections: approved creates are links (historical once a sever passes); open creates
/// are pending; open severs ride on their link; the rest are past votes.
fn group(requests: Vec<RequestView>) -> LinksView {
    let (severs, creates): (Vec<_>, Vec<_>) = requests
        .into_iter()
        .partition(|r| r.mine.request.sever.is_some());
    let cites =
        |s: &RequestView, c: &RequestView| s.mine.request.sever.as_ref() == Some(&c.mine.id);
    let mut out = LinksView::default();
    let mut past = Vec::new();
    for c in creates {
        match c.status {
            SideStatus::Open => out.pending.push(c),
            SideStatus::Rejected => past.push(c),
            SideStatus::Approved => {
                let severed = severs
                    .iter()
                    .any(|s| s.status == SideStatus::Approved && cites(s, &c));
                let link = LinkView {
                    sever: None,
                    create: c,
                };
                if severed {
                    out.historical.push(link);
                } else {
                    out.active.push(link);
                }
            }
        }
    }
    for s in severs {
        let open_on = (s.status == SideStatus::Open)
            .then(|| out.active.iter_mut().find(|l| cites(&s, &l.create)))
            .flatten();
        match open_on {
            Some(link) if link.sever.is_none() => link.sever = Some(s),
            _ if s.status == SideStatus::Open => {}
            _ => past.push(s),
        }
    }
    past.sort_by_key(|r| std::cmp::Reverse(r.mine.created_at));
    out.past = past;
    out
}
