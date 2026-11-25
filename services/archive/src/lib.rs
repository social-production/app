//! The feed index any node holding the commons can build (SEEDLESS phase B, ADR 0031): the
//! archivist's round over every listed space, the people index, and the queries the seed
//! node's `/index/*`, `/map` and `/region` routes answer. The gateway uses it now; the
//! desktop app runs it next (B2), so the feed no longer needs the seed node.

use std::fmt::Debug;
use std::future::Future;

use holochain_types::prelude::CellId;
use serde::de::DeserializeOwned;
use serde::Serialize;

mod cards;
mod event_time;
mod index;
mod markers;
mod next;
mod people;
mod region;
mod round;
mod spaces;

pub use event_time::utc_micros;
pub use index::{ProjectCard, ProjectIndex, Who};
pub use markers::{Marker, MarkerQuery};
pub use people::{people_round, PeopleIndex, PersonCard};
pub use region::{RegionItem, RegionQuery};
pub use round::{archive_round, Cells};
pub use spaces::{space_of_seed, Scoped};

/// Zome calls as the archivist's own agent: the gateway's embedded conductor now, the
/// desktop's conductor next.
pub trait Calls: Sync {
    fn call<I, O>(
        &self,
        cell: &CellId,
        zome: &str,
        func: &str,
        payload: I,
    ) -> impl Future<Output = anyhow::Result<O>> + Send
    where
        I: Serialize + Debug + Send,
        O: DeserializeOwned + Debug + Send;
}
