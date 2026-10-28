//! Which space's chat a call is about (ADR 0065), as `space_scope` does for the project zomes.
//! A private clone's calls may leave the space out; the project zome then names its one space.

use std::cell::RefCell;

use hdk::prelude::*;
use serde::de::DeserializeOwned;

use crate::guest;

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

/// Start a call about its space and return the input.
pub fn enter<T>(call: In<T>) -> ExternResult<T> {
    let space = match call.space {
        Some(space) => space,
        None => project("space")?,
    };
    CURRENT.with(|c| *c.borrow_mut() = Some(space));
    Ok(call.input)
}

/// The space this call is about.
pub fn current() -> ExternResult<ActionHash> {
    CURRENT
        .with(|c| c.borrow().clone())
        .ok_or_else(|| guest("No space chosen for this call."))
}

/// Mark the space active this hour, so indexes notice the new message.
pub fn touch() -> ExternResult<()> {
    project("touch")
}

fn project<O: DeserializeOwned + std::fmt::Debug>(name: &str) -> ExternResult<O> {
    let space = CURRENT.with(|c| c.borrow().clone());
    match call(
        CallTargetCell::Local,
        ZomeName::from("project"),
        name.into(),
        None,
        In { space, input: () },
    )? {
        ZomeCallResponse::Ok(io) => io.decode().map_err(|e| wasm_error!(e)),
        other => Err(guest(&format!("project/{name}: {other:?}"))),
    }
}
