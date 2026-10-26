import type { ChatAttachment } from '#lib/domain/chat.ts';
import { callCell, conversationCell, stringToHash, type ActionHash } from '#lib/holo/index.ts';

import type { AttachmentWire } from './chat.ts';
import { cellFor, type ProjectRef } from './projects.ts';

/** sp_attachment::CHUNK_MAX: each part of a file is its own entry (ADR 0049). */
const CHUNK_MAX = 1024 * 1024;

/** Where a chat keeps its file parts: a direct/group chat's cell, or a project's (ADR 0056). */
interface Parts {
  upload: (bytes: Uint8Array) => Promise<ActionHash>;
  get: (id: ActionHash) => Promise<Uint8Array | null>;
}

function chatParts(seed: string): Parts {
  const call = async <T>(fn: string, payload: unknown): Promise<T> =>
    callCell<T>(await conversationCell(seed), 'messages', fn, payload);
  return { upload: (b) => call('upload_chunk', b), get: (id) => call('get_chunk', id) };
}

function projectParts(project: ProjectRef): Parts {
  const call = async <T>(fn: string, payload: unknown): Promise<T> =>
    callCell<T>(await cellFor(project), 'discussion', fn, payload);
  return { upload: (b) => call('upload_chunk', b), get: (id) => call('get_chunk', id) };
}

/** web's photo/file split: JPEG, PNG and WebP show in the bubble, anything else downloads. */
const PHOTO_TYPES = new Set(['image/jpeg', 'image/png', 'image/webp']);

/**
 * Write each file's parts, one call at a time (the chain takes one write at a time), and
 * describe them for the message that follows.
 */
async function upload(parts: Parts, files: File[]): Promise<AttachmentWire[]> {
  const out: AttachmentWire[] = [];
  for (const file of files) {
    const bytes = new Uint8Array(await file.arrayBuffer());
    const chunks: ActionHash[] = [];
    for (let at = 0; at < bytes.length; at += CHUNK_MAX) {
      chunks.push(await parts.upload(bytes.slice(at, at + CHUNK_MAX)));
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
async function blob(parts: Parts, a: ChatAttachment): Promise<Blob> {
  const read: Uint8Array[] = [];
  for (const id of a.chunks) {
    const part = await parts.get(stringToHash(id));
    if (!part) throw new Error('This file is still on its way. Try again in a moment.');
    read.push(part);
  }
  return new Blob(read as BlobPart[], { type: a.mime });
}

export const uploadFiles = (seed: string, files: File[]) => upload(chatParts(seed), files);
export const attachmentBlob = (seed: string, a: ChatAttachment) => blob(chatParts(seed), a);
export const uploadProjectFiles = (project: ProjectRef, files: File[]) =>
  upload(projectParts(project), files);
export const projectAttachmentBlob = (project: ProjectRef, a: ChatAttachment) =>
  blob(projectParts(project), a);
