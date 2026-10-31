import type { CallLine, ChatAttachment, ChatKindOfLine, ChatMessage } from '#lib/domain/chat.ts';
import {
  callCell,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { uploadProjectFiles } from './attachments.ts';
import { cellFor, type ProjectRef } from './projects.ts';

export interface MessageWire {
  id: ActionHash;
  author: AgentPubKey;
  body: string;
  created_at: number;
  reply_to: {
    id: ActionHash;
    author: AgentPubKey | null;
    preview: string;
    deleted: boolean;
  } | null;
  kind?: ChatKindOfLine;
  edited?: boolean;
  pinned?: boolean;
  attachments?: AttachmentWire[];
  call?: CallLine | null;
}

export interface AttachmentWire {
  kind: 'image' | 'file';
  name: string;
  mime: string;
  size: number;
  chunks: ActionHash[];
}

function toAttachment(a: AttachmentWire): ChatAttachment {
  return {
    kind: a.kind,
    name: a.name,
    mime: a.mime,
    size: a.size,
    chunks: a.chunks.map(hashToString),
  };
}

export type { ProjectRef };

const discussion = async <T>(project: ProjectRef, fn: string, payload?: unknown): Promise<T> =>
  callCell<T>(await cellFor(project), 'discussion', fn, payload);

/** A project's chain takes one write at a time: chat writes (and read marks) wait their turn. */
let tail: Promise<unknown> = Promise.resolve();
export function queued<T>(write: () => Promise<T>): Promise<T> {
  const run = tail.then(write, write);
  tail = run.catch(() => undefined);
  return run;
}

/** The project chat's newest messages, oldest first (the zome returns at most 200). */
export async function listMessages(project: ProjectRef): Promise<ChatMessage[]> {
  const wire = await discussion<MessageWire[]>(project, 'list_messages');
  return wire.map(toChatMessage);
}

/** Project chats and direct/group chats return messages in the same shape. */
export function toChatMessage(m: MessageWire): ChatMessage {
  return {
    id: hashToString(m.id),
    author: hashToString(m.author),
    body: m.body,
    createdAt: m.created_at,
    replyTo: m.reply_to && {
      id: hashToString(m.reply_to.id),
      author: m.reply_to.author ? hashToString(m.reply_to.author) : null,
      preview: m.reply_to.preview,
      deleted: m.reply_to.deleted,
    },
    kind: m.kind ?? 'text',
    edited: m.edited ?? false,
    pinned: m.pinned ?? false,
    attachments: (m.attachments ?? []).map(toAttachment),
    call: m.call ?? null,
  };
}

/** Anyone signed in may write in a project's chat, as on web (ADR 0038); photos and files
 * ride along since ADR 0056. */
export async function postMessage(
  project: ProjectRef,
  body: string,
  replyTo: string | null,
  files: File[] = [],
): Promise<string> {
  const id = await queued(async () => {
    const attachments = files.length ? await uploadProjectFiles(project, files) : [];
    return discussion<ActionHash>(project, 'post_message', {
      body,
      reply_to: replyTo ? stringToHash(replyTo) : null,
      attachments,
    });
  });
  return hashToString(id);
}

/** How far you've read this project's chat, kept privately on your chain (ADR 0056). */
export async function chatReadAt(project: ProjectRef): Promise<number | null> {
  return discussion<number | null>(project, 'read_at', null);
}

/** Never backwards; a mark that loses a race with another write is simply retried later. */
export async function markChatRead(project: ProjectRef, at: number): Promise<void> {
  await queued(() => discussion<null>(project, 'mark_read', at)).catch(() => undefined);
}

/** Only the author can delete a message; it goes for everyone. */
export async function deleteMessage(project: ProjectRef, id: string): Promise<void> {
  await queued(() => discussion<null>(project, 'delete_message', stringToHash(id)));
}
