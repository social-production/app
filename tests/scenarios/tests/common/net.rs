//! A project network of `n` agents, each with a directory and a project cell (governance and
//! plan scenarios).

use holochain::prelude::*;
use holochain::sweettest::{await_consistency, SweetCell, SweetConductorBatch};
use serde::{Deserialize, Serialize};

/// Mirrors `project_integrity::Charter` (tests can't link two integrity crates).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Charter {
    pub title: String,
    pub description: String,
    pub mode: String,
    pub subtype: String,
    pub location_label: String,
    pub phase: String,
    pub tags: Vec<String>,
}

pub fn charter(mode: &str) -> Charter {
    Charter {
        title: "Garden".into(),
        description: "Growing food together.".into(),
        mode: mode.into(),
        subtype: "standard".into(),
        location_label: String::new(),
        phase: "Proposal".into(),
        tags: Vec::new(),
    }
}

pub struct Net {
    pub conductors: SweetConductorBatch,
    pub cells: Vec<(SweetCell, SweetCell)>,
}

impl Net {
    /// `n` agents; the first `founders` are Full. Agent 0 creates a productive project; all
    /// others join.
    pub async fn new(n: usize, founders: usize, seed: &str) -> Self {
        Self::with_charter(n, founders, seed, charter("productive")).await
    }

    pub async fn with_charter(n: usize, founders: usize, seed: &str, charter: Charter) -> Self {
        let mut conductors = SweetConductorBatch::standard(n).await;
        let cells = super::project_with_directory(&mut conductors, founders, seed).await;
        let net = Self { conductors, cells };
        let _: ActionHash = net.call(0, "project", "create_charter", charter).await;
        for i in 1..n {
            let _: () = net.call(i, "project", "join", ()).await;
        }
        net.settle().await;
        net
    }

    pub async fn call<I, O>(&self, i: usize, zome: &str, func: &str, payload: I) -> O
    where
        I: Serialize + std::fmt::Debug,
        O: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        Box::pin(self.conductors[i].call(&self.cells[i].1.zome(zome), func, payload)).await
    }

    pub async fn try_call<I, O>(
        &self,
        i: usize,
        zome: &str,
        func: &str,
        payload: I,
    ) -> Result<O, String>
    where
        I: Serialize + std::fmt::Debug,
        O: serde::de::DeserializeOwned + std::fmt::Debug,
    {
        Box::pin(self.conductors[i].call_fallible(&self.cells[i].1.zome(zome), func, payload))
            .await
            .map_err(|e| format!("{e:?}"))
    }

    /// Stop every conductor and wait for it (see `common::shut_down`).
    pub async fn shut_down(mut self) {
        super::shut_down(&mut self.conductors).await;
    }

    /// Wait for each network on its own: cells of different DNAs never share data.
    pub async fn settle(&self) {
        let directory: Vec<&SweetCell> = self.cells.iter().map(|(d, _)| d).collect();
        let project: Vec<&SweetCell> = self.cells.iter().map(|(_, p)| p).collect();
        await_consistency(directory)
            .await
            .expect("directory consistency");
        await_consistency(project)
            .await
            .expect("project consistency");
    }

    /// Every agent signals demand, so the project may leave Proposal.
    pub async fn demand_all(&self) {
        for i in 0..self.cells.len() {
            let _: () = self.call(i, "project", "set_signal", Some("Demand")).await;
        }
        self.settle().await;
    }
}
