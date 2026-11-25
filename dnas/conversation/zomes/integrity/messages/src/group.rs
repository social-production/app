//! Co-managers and a group's photo (ADR 0122).

use hdi::prelude::*;

/// A manager makes `agent` a manager too (`on`), or stops them being one (ADR 0122).
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Manager {
    pub agent: AgentPubKey,
    pub on: bool,
    pub by: Option<ActionHash>,
}

/// A group's photo, already cropped and shrunk by the app; the newest by a manager at the
/// time counts (ADR 0122).
#[hdk_entry_helper]
#[derive(Clone, PartialEq)]
pub struct Picture {
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
    pub by: Option<ActionHash>,
}

/// The largest group photo (a 512 px JPEG is well under it).
pub const PICTURE_MAX: usize = 96 * 1024;
