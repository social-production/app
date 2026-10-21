//! Two projects for the link scenarios (ADR 0042): each founder created one, and both have
//! a cell of each (opening a project clones it).

use holo_hash::AgentPubKeyB64;
use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetAgents, SweetCell, SweetConductorBatch};
use serde::{Deserialize, Serialize};
use sp_governance::links::SideStatus;
use sp_governance::Verdict;

use super::net::charter;

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

/// Two Full founders; founder 0 created and joined project A, founder 1 project B. Both
/// have a cell of both projects (opening a project clones it).
pub struct Pair {
    pub conductors: SweetConductorBatch,
    /// `cells[agent][project]`.
    pub cells: Vec<[SweetCell; 2]>,
    pub refs: [ProjectRef; 2],
}

impl Pair {
    /// Stop every conductor and wait for it (see `common::shut_down`).
    pub async fn shut_down(mut self) {
        super::shut_down(&mut self.conductors).await;
    }

    pub async fn new() -> Self {
        let mut conductors = SweetConductorBatch::standard(2).await;
        let mut agents = Vec::new();
        for i in 0..2 {
            agents.push(SweetAgents::one(conductors[i].keystore()).await);
        }
        let directory = super::directory_with_founders(&agents).await;
        let seeds: Vec<String> = ["a", "b"]
            .iter()
            .zip(&agents)
            .map(|(s, a)| format!("{}.{s}", AgentPubKeyB64::from(a.clone())))
            .collect();
        let mut projects = Vec::new();
        for seed in &seeds {
            let modifiers =
                DnaModifiersOpt::<SerializedBytes>::none().with_network_seed(seed.clone());
            projects.push(super::project_dna_with(modifiers).await);
        }
        let mut cells = Vec::new();
        for (i, agent) in agents.into_iter().enumerate() {
            let roles = [
                ("directory".to_string(), directory.clone()),
                ("project".to_string(), projects[0].clone()),
                ("project_b".to_string(), projects[1].clone()),
            ];
            let app = conductors[i]
                .setup_app_for_agent("sp", agent, &roles)
                .await
                .expect("install app");
            let mut by_dna = app.into_cells();
            let mut take = |dna: &DnaHash| {
                let at = by_dna.iter().position(|c| c.cell_id().dna_hash() == dna);
                by_dna.remove(at.expect("cell"))
            };
            cells.push([take(projects[0].dna_hash()), take(projects[1].dna_hash())]);
        }
        conductors.exchange_peer_info().await;
        let refs = [0u8, 1].map(|i| ProjectRef {
            listing: ActionHash::from_raw_36(vec![i; 36]),
            network_seed: seeds[usize::from(i)].clone(),
        });
        let pair = Self {
            conductors,
            cells,
            refs,
        };
        for p in 0..2 {
            let _: ActionHash = pair
                .call(p, p, "project", "create_charter", charter("productive"))
                .await;
        }
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
        let zome = self.cells[agent][project].zome(zome);
        Box::pin(self.conductors[agent].call(&zome, func, payload)).await
    }

    pub async fn refused(&self, agent: usize, project: usize, request: LinkRequest, why: &str) {
        let zome = self.cells[agent][project].zome("governance");
        let kind = ProposalKind::Link(request);
        let result: Result<ActionHash, _> =
            Box::pin(self.conductors[agent].call_fallible(&zome, "propose", kind)).await;
        assert!(result.is_err(), "{why}");
    }

    pub async fn settle(&self) {
        for p in 0..2 {
            let cells: Vec<&SweetCell> = self.cells.iter().map(|c| &c[p]).collect();
            await_consistency(cells).await.expect("project consistency");
        }
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
