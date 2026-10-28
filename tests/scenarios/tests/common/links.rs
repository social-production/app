//! Two projects for the link scenarios (ADR 0042): each founder founded one space in the
//! commons (ADR 0065), so both live in every agent's one project cell.

use holo_hash::{ActionHashB64, AgentPubKeyB64};
use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetCell, SweetConductorBatch};
use serde::{Deserialize, Serialize};
use sp_governance::links::SideStatus;
use sp_governance::Verdict;

use super::net::{charter, Scoped};

/// Mirrors `project_integrity::ProjectRef` and `LinkRequest`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ProjectRef {
    pub listing: ActionHash,
    pub network_seed: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LinkRequest {
    pub other: ProjectRef,
    pub relationship: String,
    pub summary: String,
    pub sever: Option<ActionHash>,
    pub source: Option<ActionHash>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ProposalKind {
    Link(LinkRequest),
}

/// Mirrors `governance::TallyView`, `HalfView` and `LinkSides`, so sides pass back unchanged.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TallyView {
    pub yes: u32,
    pub no: u32,
    pub neutral: u32,
    pub population: u32,
    pub required: u32,
    pub status: Verdict,
    pub closed_at: Option<Timestamp>,
    pub my_vote: Option<String>,
    pub settled: bool,
    pub added: u32,
    pub missing: u32,
    pub challenge_until: Option<Timestamp>,
    pub can_challenge: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HalfView {
    pub id: ActionHash,
    pub author: AgentPubKey,
    pub created_at: Timestamp,
    pub request: LinkRequest,
    pub tally: TallyView,
    pub status: SideStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkSides {
    pub network_seed: String,
    pub am_member: bool,
    pub halves: Vec<HalfView>,
}

#[derive(Deserialize, Debug)]
pub struct RequestView {
    pub mine: HalfView,
    pub theirs: Option<HalfView>,
    pub status: SideStatus,
}

#[derive(Deserialize, Debug)]
pub struct LinkView {
    pub create: RequestView,
    pub sever: Option<RequestView>,
}

#[derive(Deserialize, Debug)]
pub struct LinksView {
    pub active: Vec<LinkView>,
    pub historical: Vec<LinkView>,
    pub pending: Vec<RequestView>,
    pub past: Vec<RequestView>,
}

#[derive(Serialize, Debug)]
pub struct VoteInput {
    pub proposal: ActionHash,
    pub choice: &'static str,
}

/// Two Full founders in one commons (ADR 0065); founder 0 founded space A, founder 1 space B.
pub struct Pair {
    pub conductors: SweetConductorBatch,
    /// Each agent's commons cell (both spaces live in it).
    pub cells: Vec<SweetCell>,
    pub spaces: [ActionHash; 2],
    pub refs: [ProjectRef; 2],
}

impl Pair {
    /// Stop every conductor and wait for it (see `common::shut_down`).
    pub async fn shut_down(self) {
        super::shut_down(self.conductors).await;
    }

    pub async fn new() -> Self {
        let mut conductors = SweetConductorBatch::standard(2).await;
        let seed = format!("{}links", super::COMMONS);
        let cells: Vec<SweetCell> = super::project_with_directory(&mut conductors, 2, &seed)
            .await
            .into_iter()
            .map(|(_, project)| project)
            .collect();
        let mut spaces = Vec::new();
        for (i, cell) in cells.iter().enumerate() {
            let space: ActionHash = Box::pin(conductors[i].call(
                &cell.zome("project"),
                "create_charter",
                charter("productive"),
            ))
            .await;
            spaces.push(space);
        }
        let spaces: [ActionHash; 2] = [spaces[0].clone(), spaces[1].clone()];
        let refs = [0u8, 1].map(|i| {
            let p = usize::from(i);
            ProjectRef {
                listing: ActionHash::from_raw_36(vec![i; 36]),
                network_seed: format!(
                    "{}.space.{}",
                    AgentPubKeyB64::from(cells[p].agent_pubkey().clone()),
                    ActionHashB64::from(spaces[p].clone())
                ),
            }
        });
        let pair = Self {
            conductors,
            cells,
            spaces,
            refs,
        };
        pair.settle().await;
        pair
    }

    pub async fn call<I, O>(
        &self,
        agent: usize,
        project: usize,
        zome: &str,
        func: &str,
        payload: I,
    ) -> O
    where
        I: Serialize + std::fmt::Debug,
        O: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        let zome = self.cells[agent].zome(zome);
        let input = Scoped {
            space: self.spaces[project].clone(),
            input: payload,
        };
        Box::pin(self.conductors[agent].call(&zome, func, input)).await
    }

    pub async fn refused(&self, agent: usize, project: usize, request: LinkRequest, why: &str) {
        let zome = self.cells[agent].zome("governance");
        let input = Scoped {
            space: self.spaces[project].clone(),
            input: ProposalKind::Link(request),
        };
        let result: Result<ActionHash, _> =
            Box::pin(self.conductors[agent].call_fallible(&zome, "propose", input)).await;
        assert!(result.is_err(), "{why}");
    }

    pub async fn settle(&self) {
        await_consistency(&self.cells)
            .await
            .expect("commons consistency");
    }

    /// `agent` files a request in `from` and mirrors it into the other project, as the UI does.
    pub async fn request(
        &self,
        agent: usize,
        from: usize,
        sever: [Option<ActionHash>; 2],
    ) -> ActionHash {
        let to = 1 - from;
        let mut request = link(&self.refs[to], "Shared tools");
        request.sever.clone_from(&sever[from]);
        let id: ActionHash = self
            .call(
                agent,
                from,
                "governance",
                "propose",
                ProposalKind::Link(request.clone()),
            )
            .await;
        request.other = self.refs[from].clone();
        request.sever.clone_from(&sever[to]);
        request.source = Some(id.clone());
        let _: ActionHash = self
            .call(
                agent,
                to,
                "governance",
                "propose",
                ProposalKind::Link(request),
            )
            .await;
        self.settle().await;
        id
    }

    pub async fn vote(
        &self,
        agent: usize,
        project: usize,
        half: &ActionHash,
        choice: &'static str,
    ) {
        let input = VoteInput {
            proposal: half.clone(),
            choice,
        };
        let _: ActionHash = self.call(agent, project, "governance", "vote", input).await;
        self.settle().await;
    }

    /// Project `p`'s Links tab, as its member sees it.
    pub async fn links(&self, p: usize) -> LinksView {
        let other: LinkSides = self.call(p, 1 - p, "governance", "link_sides", ()).await;
        self.call(p, p, "governance", "list_links", vec![other])
            .await
    }
}

pub fn link(other: &ProjectRef, summary: &str) -> LinkRequest {
    LinkRequest {
        other: other.clone(),
        relationship: String::new(),
        summary: summary.into(),
        sever: None,
        source: None,
    }
}
