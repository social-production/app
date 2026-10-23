//! Photos and files on messages (ADR 0049): `upload_chunk` writes one part of a file before
//! the message that lists it, and `get_chunk` reads a part back for the UI to assemble.

use hdk::prelude::*;
use messages_integrity::{is_type, Attachment, Chunk, EntryTypes, UnitEntryTypes};
use sp_attachment::{check_attachment, check_chunk};

use crate::{guest, people, reads};

/// Bytes as msgpack `bin` (a `Uint8Array` in the UI), not a list of numbers.
#[derive(Serialize, Deserialize, Debug)]
pub struct Bytes(#[serde(with = "serde_bytes")] pub Vec<u8>);

/// One part of a file you're about to send (at most 1 MB); its hash goes in the message.
#[hdk_extern]
pub fn upload_chunk(bytes: Bytes) -> ExternResult<ActionHash> {
    check_chunk(bytes.0.len()).map_err(|reason| guest(&reason))?;
    let chat = people::load()?;
    chat.require_member()?;
    let invite = chat.my_invite()?;
    create_entry(EntryTypes::Chunk(Chunk {
        bytes: bytes.0,
        invite,
    }))
}

/// A part of a file in this chat; `None` while it can't be reached.
#[hdk_extern]
pub fn get_chunk(hash: ActionHash) -> ExternResult<Option<Bytes>> {
    let Some(record) = reads::record(&hash)? else {
        return Ok(None);
    };
    if !is_type(record.action(), UnitEntryTypes::Chunk)? {
        return Err(guest("That is not part of a file."));
    }
    Ok(record
        .entry()
        .to_app_option::<Chunk>()
        .map_err(|e| wasm_error!(e))?
        .map(|c| Bytes(c.bytes)))
}

/// The same rules as validation, before anything is written, so the sender sees web's words.
pub fn check(attachments: &[Attachment]) -> ExternResult<()> {
    for a in attachments {
        let size = usize::try_from(a.size).unwrap_or(usize::MAX);
        check_attachment(&a.kind, &a.name, &a.mime, size, a.chunks.len())
            .map_err(|reason| guest(&reason))?;
    }
    Ok(())
}

/// The chunks of `id`, one of the caller's own messages `mine`, deleted with it.
pub fn chunks_of(mine: &[Record], id: &ActionHash) -> ExternResult<Vec<ActionHash>> {
    let Some(record) = mine.iter().find(|r| r.action_address() == id) else {
        return Ok(Vec::new());
    };
    Ok(reads::decode(record)?
        .map(|m| m.attachments.into_iter().flat_map(|a| a.chunks).collect())
        .unwrap_or_default())
}
