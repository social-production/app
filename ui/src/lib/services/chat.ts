import type { ChatAttachment, ChatMessage } from '#lib/domain/chat.ts';
import {
  callCell,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

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
  kind?: 'text' | 'system';
  edited?: boolean;
  pinned?: boolean;
  attachments?: AttachmentWire[];
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
  };
}

/** Anyone signed in may write in a project's chat, as on web (ADR 0038). */
export async function postMessage(
  project: ProjectRef,
  body: string,
  replyTo: string | null,
): Promise<string> {
  const id = await discussion<ActionHash>(project, 'post_message', {
    body,
    reply_to: replyTo ? stringToHash(replyTo) : null,
  });
  return hashToString(id);
}

/** Only the author can delete a message; it goes for everyone. */
export async function deleteMessage(project: ProjectRef, id: string): Promise<void> {
  await discussion<null>(project, 'delete_message', stringToHash(id));
}
