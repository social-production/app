//! A project space shared by `n` agents, each with a directory and a project cell (governance
//! and plan scenarios). Public spaces live in the commons (ADR 0065): the cell holds any number
//! of spaces, so every call names its space. Private ones (seeds with `private.`/`organized.`)
//! are clones holding one space, whose calls may leave it out.

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
    /// The space every call is about; `None` in a private clone (its one space).
    pub space: Option<ActionHash>,
}

/// A call's input with its space, as the coordinators' `In<T>` reads it.
#[derive(Serialize, Debug)]
pub struct Scoped<T> {
    pub space: ActionHash,
    pub input: T,
}

/// Calls that aren't about one space: founding one, and file parts.
const UNSCOPED: [&str; 3] = ["create_charter", "upload_chunk", "get_chunk"];

/// A private seed (ADR 0061) makes a clone; anything else is a space in the commons.
fn is_private(seed: &str) -> bool {
    seed.starts_with("private.") || seed.starts_with("organized.")
}

impl Net {
    /// `n` agents; the first `founders` are Full. Agent 0 creates a productive project; all
    /// others join.
    pub async fn new(n: usize, founders: usize, seed: &str) -> Self {
        Self::with_charter(n, founders, seed, charter("productive")).await
    }

    pub async fn with_charter(n: usize, founders: usize, seed: &str, charter: Charter) -> Self {
        let mut conductors = SweetConductorBatch::standard(n).await;
        let seed = if is_private(seed) {
            seed.to_string()
        } else {
            format!("{}{seed}", super::COMMONS)
        };
        let cells = super::project_with_directory(&mut conductors, founders, &seed).await;
        let mut net = Self {
            conductors,
            cells,
            space: None,
        };
        let space: ActionHash = net.call(0, "project", "create_charter", charter).await;
        if !is_private(&seed) {
            net.space = Some(space);
        }
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
        let zome = self.cells[i].1.zome(zome);
        match self.scope(func) {
            Some(space) => {
                let payload = Scoped {
                    space,
                    input: payload,
                };
                Box::pin(self.conductors[i].call(&zome, func, payload)).await
            }
            None => Box::pin(self.conductors[i].call(&zome, func, payload)).await,
        }
    }

    /// The space to name in a call to `func`, if any.
    fn scope(&self, func: &str) -> Option<ActionHash> {
        self.space.clone().filter(|_| !UNSCOPED.contains(&func))
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
        let zome = self.cells[i].1.zome(zome);
        let result = match self.scope(func) {
            Some(space) => {
                let payload = Scoped {
                    space,
                    input: payload,
                };
                Box::pin(self.conductors[i].call_fallible(&zome, func, payload)).await
            }
            None => Box::pin(self.conductors[i].call_fallible(&zome, func, payload)).await,
        };
        result.map_err(|e| format!("{e:?}"))
    }

    /// Stop every conductor and wait for it (see `common::shut_down`).
    pub async fn shut_down(self) {
        super::shut_down(self.conductors).await;
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
