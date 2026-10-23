import type { ChatAttachment } from '#lib/domain/chat.ts';
import { callCell, conversationCell, stringToHash, type ActionHash } from '#lib/holo/index.ts';

import type { AttachmentWire } from './chat.ts';

/** sp_attachment::CHUNK_MAX: each part of a file is its own entry (ADR 0049). */
const CHUNK_MAX = 1024 * 1024;

const chat = async <T>(seed: string, fn: string, payload?: unknown): Promise<T> =>
  callCell<T>(await conversationCell(seed), 'messages', fn, payload);

/** web's photo/file split: JPEG, PNG and WebP show in the bubble, anything else downloads. */
const PHOTO_TYPES = new Set(['image/jpeg', 'image/png', 'image/webp']);

/**
 * Write each file's parts, one call at a time (the chain takes one write at a time), and
 * describe them for the message that follows.
 */
export async function uploadFiles(seed: string, files: File[]): Promise<AttachmentWire[]> {
  const out: AttachmentWire[] = [];
  for (const file of files) {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const chunks: ActionHash[] = [];
    for (let at = 0; at < bytes.length; at += CHUNK_MAX) {
      chunks.push(await chat<ActionHash>(seed, 'upload_chunk', bytes.slice(at, at + CHUNK_MAX)));
    }
    const mime = (file.type || 'application/octet-stream').toLowerCase();
    out.push({
      kind: PHOTO_TYPES.has(mime) ? 'image' : 'file',
      name: file.name || 'file',
      mime,
      size: bytes.length,
      chunks,
    });
  }
  return out;
}

/** The attachment's bytes, read part by part; throws while a part can't be reached. */
export async function attachmentBlob(seed: string, a: ChatAttachment): Promise<Blob> {
  const parts: Uint8Array[] = [];
  for (const id of a.chunks) {
    const part = await chat<Uint8Array | null>(seed, 'get_chunk', stringToHash(id));
    if (!part) throw new Error('This file is still on its way. Try again in a moment.');
    parts.push(part);
  }
  return new Blob(parts as BlobPart[], { type: a.mime });
}
