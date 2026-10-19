import type { ChatMessage } from '#lib/domain/chat.ts';
import {
  callCell,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { cellOf } from './projects.ts';

interface MessageWire {
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
}

const discussion = async <T>(projectId: string, fn: string, payload?: unknown): Promise<T> => {
  const { cell } = await cellOf(projectId);
  return callCell<T>(cell, 'discussion', fn, payload);
};

/** The project chat's newest messages, oldest first (the zome returns at most 200). */
export async function listMessages(projectId: string): Promise<ChatMessage[]> {
  const wire = await discussion<MessageWire[]>(projectId, 'list_messages');
  return wire.map((m) => ({
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
  }));
}

/** Anyone signed in may write in a project's chat, as on web (ADR 0038). */
export async function postMessage(
  projectId: string,
  body: string,
  replyTo: string | null,
): Promise<string> {
  const id = await discussion<ActionHash>(projectId, 'post_message', {
    body,
    reply_to: replyTo ? stringToHash(replyTo) : null,
  });
  return hashToString(id);
}

/** Only the author can delete a message; it goes for everyone. */
export async function deleteMessage(projectId: string, id: string): Promise<void> {
  await discussion<null>(projectId, 'delete_message', stringToHash(id));
}
