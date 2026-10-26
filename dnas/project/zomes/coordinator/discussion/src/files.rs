//! Photos and files in the project chat (ADR 0056, as ADR 0049 for direct chats):
//! `upload_chunk` writes one part before the message that lists it, `get_chunk` reads one back.

use discussion_integrity::{is_type, Attachment, ChatMessage, Chunk, EntryTypes, UnitEntryTypes};
use hdk::prelude::*;
use sp_attachment::{check_attachment, check_chunk};

use crate::{guest, reads};

/// Bytes as msgpack `bin` (a `Uint8Array` in the UI), not a list of numbers.
#[derive(Serialize, Deserialize, Debug)]
pub struct Bytes(#[serde(with = "serde_bytes")] pub Vec<u8>);

pub fn upload(bytes: Bytes) -> ExternResult<ActionHash> {
    check_chunk(bytes.0.len()).map_err(|reason| guest(&reason))?;
    create_entry(EntryTypes::Chunk(Chunk { bytes: bytes.0 }))
}

pub fn chunk(hash: ActionHash) -> ExternResult<Option<Bytes>> {
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

/// The chunks of one of the caller's own messages, deleted with it.
pub fn chunks_of(mine: &[Record], id: &ActionHash) -> ExternResult<Vec<ActionHash>> {
    let Some(record) = mine.iter().find(|r| r.action_address() == id) else {
        return Ok(Vec::new());
    };
    Ok(record
        .entry()
        .to_app_option::<ChatMessage>()
        .map_err(|e| wasm_error!(e))?
        .map(|m| m.attachments.into_iter().flat_map(|a| a.chunks).collect())
        .unwrap_or_default())
}
