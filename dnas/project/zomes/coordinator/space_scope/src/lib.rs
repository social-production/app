//! Which space a call is about (ADR 0065). Public spaces live together in the commons (the
//! project DNA's base cell), so every call names its space: the charter's create hash. A
//! private space is a clone holding one space, so its calls may leave it out.
//!
//! Each zome call runs in its own WASM instance, so `enter` at the top of an extern sets the
//! space for the rest of that call; calls to the DNA's other zomes pass it on (`pass`).

use std::cell::RefCell;

use hdk::prelude::*;
use holo_hash::{ActionHashB64, AgentPubKeyB64};
use project_integrity::{creator, space_creator, LinkTypes, UnitEntryTypes, SPACE_SEED_WORD};

/// A call's input with the space it is about. A bare input (no `{space, input}` around it)
/// means the clone's one space, so private-space callers need not name it.
#[derive(Serialize, Debug, Clone)]
pub struct In<T> {
    /// `None` in a private clone: its one space.
    pub space: Option<ActionHash>,
    pub input: T,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Wire<T> {
    Scoped {
        #[serde(default)]
        space: Option<ActionHash>,
        input: T,
    },
    Bare(T),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for In<T> {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        Ok(match Wire::deserialize(d)? {
            Wire::Scoped { space, input } => Self { space, input },
            Wire::Bare(input) => Self { space: None, input },
        })
    }
}

thread_local! {
    static CURRENT: RefCell<Option<ActionHash>> = const { RefCell::new(None) };
}

#[must_use]
pub fn guest(message: &str) -> WasmError {
    wasm_error!(WasmErrorInner::Guest(message.into()))
}

/// Start a call about `space` (or the clone's one space) and return the input.
pub fn enter<T>(call: In<T>) -> ExternResult<T> {
    let space = match call.space {
        Some(space) => space,
        None => {
            clone_space()?.ok_or_else(|| guest("Which project? This space has no charter yet."))?
        }
    };
    CURRENT.with(|c| *c.borrow_mut() = Some(space));
    Ok(call.input)
}

/// Start a call about a space already known (a charter just written).
pub fn set(space: ActionHash) {
    CURRENT.with(|c| *c.borrow_mut() = Some(space));
}

/// The space this call is about.
pub fn current() -> ExternResult<ActionHash> {
    CURRENT
        .with(|c| c.borrow().clone())
        .ok_or_else(|| guest("No space chosen for this call."))
}

/// The base every link of the space hangs off: the charter hash.
pub fn base() -> ExternResult<AnyLinkableHash> {
    Ok(current()?.into())
}

/// `input` wrapped for a call to another zome of this DNA, about the same space.
pub fn pass<T>(input: T) -> ExternResult<In<T>> {
    Ok(In {
        space: Some(current()?),
        input,
    })
}

/// The current space's seed as directory listings and links name it: a private clone's own
/// network seed, or `<creator>.space.<id>` for a space in the commons.
pub fn seed() -> ExternResult<String> {
    if creator()?.is_some() {
        return Ok(dna_info()?.modifiers.network_seed);
    }
    let space = current()?;
    let founder = space_creator(&space)?.ok_or_else(|| guest("That isn't a project."))?;
    Ok(format!(
        "{}.{SPACE_SEED_WORD}.{}",
        AgentPubKeyB64::from(founder),
        ActionHashB64::from(space)
    ))
}

/// "charter" anchor: how a private clone finds its one space.
pub const CHARTER: &str = "charter";

/// The clone's one space: its charter, from the own chain first, then the "charter" anchor.
pub fn clone_space() -> ExternResult<Option<ActionHash>> {
    let charters = EntryType::App(UnitEntryTypes::Charter.try_into()?);
    let own = query(
        ChainQueryFilter::new()
            .entry_type(charters)
            .action_type(ActionType::Create),
    )?;
    if let Some(first) = own.first() {
        return Ok(Some(first.action_address().clone()));
    }
    let mut found = get_links(
        LinkQuery::try_new(
            Path::from(CHARTER).path_entry_hash()?,
            LinkTypes::ProjectCharter,
        )?,
        GetStrategy::default(),
    )?;
    found.sort_by_key(|l| l.timestamp);
    Ok(found.into_iter().find_map(|l| l.target.into_action_hash()))
}

/// Mark the current space active this hour (`recent.<hour>` → space), once per writer and
/// hour, so indexes re-read only spaces that changed (ADR 0065).
pub fn touch() -> ExternResult<()> {
    let space = current()?;
    let hour = sys_time()?.as_micros() / 3_600_000_000;
    let bucket = AnyLinkableHash::from(Path::from(format!("recent.{hour}")).path_entry_hash()?);
    let scoped: ScopedLinkType = LinkTypes::Recent.try_into()?;
    let target = AnyLinkableHash::from(space.clone());
    let mine = query(ChainQueryFilter::new().action_type(ActionType::CreateLink))?;
    let done = mine.iter().any(|r| match &r.action().data {
        ActionData::CreateLink(l) => {
            l.zome_index == scoped.zome_index
                && l.link_type == scoped.zome_type
                && l.base_address == bucket
                && l.target_address == target
        }
        _ => false,
    });
    if !done {
        create_link(bucket, space, LinkTypes::Recent, ())?;
    }
    Ok(())
}

/// Spaces active in `hour` (hours since the Unix epoch), each once.
pub fn active_in(hour: i64) -> ExternResult<Vec<ActionHash>> {
    let bucket = Path::from(format!("recent.{hour}")).path_entry_hash()?;
    let mut out: Vec<ActionHash> = Vec::new();
    for link in get_links(
        LinkQuery::try_new(bucket, LinkTypes::Recent)?,
        GetStrategy::default(),
    )? {
        if let Some(space) = link.target.into_action_hash() {
            if !out.contains(&space) {
                out.push(space);
            }
        }
    }
    Ok(out)
}
