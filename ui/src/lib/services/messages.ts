import type { ChatMessage } from '#lib/domain/chat.ts';
import type { ChatKind, ChatSummary, Contact, NewChat } from '#lib/domain/messages.ts';
import {
  callCell,
  callZome,
  conversationCell,
  hashToString,
  newChatSeed,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { toChatMessage, type MessageWire } from './chat.ts';
import { listPeople, myAgent, namesOf } from './people.ts';

interface InvitationWire {
  from: AgentPubKey;
  seed: string;
  at: number;
}

interface SummaryWire {
  kind: ChatKind;
  creator: AgentPubKey;
  members: AgentPubKey[];
  title: string | null;
  last: { author: AgentPubKey; preview: string; created_at: number } | null;
  unread: number;
}

/** The base conversation cell: everyone's inbox of chat invitations (ADR 0043). */
const inbox = <T>(fn: string, payload?: unknown): Promise<T> =>
  callZome<T>('conversation', 'messages', fn, payload);

const chat = async <T>(seed: string, fn: string, payload?: unknown): Promise<T> =>
  callCell<T>(await conversationCell(seed), 'messages', fn, payload);

/** Every chat you're in, each once: their seeds and when the invitation came. */
export async function myChats(): Promise<{ seed: string; at: number }[]> {
  const wire = await inbox<InvitationWire[]>('inbox');
  return wire.map((i) => ({ seed: i.seed, at: i.at }));
}

export async function chatSummary(seed: string, invitedAt: number): Promise<ChatSummary> {
  const s = await chat<SummaryWire>(seed, 'summary');
  return {
    id: seed,
    kind: s.kind,
    creator: hashToString(s.creator),
    members: s.members.map(hashToString),
    title: s.title,
    last: s.last && {
      author: hashToString(s.last.author),
      preview: s.last.preview,
      createdAt: s.last.created_at,
    },
    unread: s.unread,
    invitedAt,
  };
}

/**
 * Start a chat (web's start_direct_conversation / create_group_conversation): clone it,
 * invite everyone, send each of them (and yourself, for your other devices) the seed through
 * the inbox, then post the first message. Returns the chat's id.
 */
export async function startChat(
  kind: ChatKind,
  members: string[],
  title: string | null,
  body: string,
): Promise<string> {
  const me = await myAgent();
  const seed = newChatSeed(me, kind);
  await chat<null>(seed, 'start_chat', { members: members.map(stringToHash), title });
  for (const to of [me, ...members]) {
    await inbox<null>('send_invite', { to: stringToHash(to), seed });
  }
  await postChatMessage(seed, body, null);
  return seed;
}

export async function listChatMessages(seed: string): Promise<ChatMessage[]> {
  return (await chat<MessageWire[]>(seed, 'list_messages')).map(toChatMessage);
}

export async function postChatMessage(
  seed: string,
  body: string,
  replyTo: string | null,
): Promise<string> {
  const id = await chat<ActionHash>(seed, 'post_message', {
    body,
    reply_to: replyTo ? stringToHash(replyTo) : null,
  });
  return hashToString(id);
}

export async function deleteChatMessage(seed: string, id: string): Promise<void> {
  await chat<null>(seed, 'delete_message', stringToHash(id));
}

/** A private read mark (web's mark_conversation_as_read); written only when needed. */
export async function markChatRead(seed: string): Promise<void> {
  await chat<null>(seed, 'mark_read');
}

/** Everyone with a profile, by name, for the compose sheet's suggestions. */
export async function contacts(): Promise<Contact[]> {
  const agents = await listPeople();
  const names = await namesOf(agents);
  return agents.map((agent) => ({ agent, name: names[agent] ?? 'Unnamed' }));
}

/**
 * Start what the compose sheet asked for, or post into the direct chat you already have with
 * that person (web's start_direct_conversation reuses it). `known` saves reading every chat
 * when the caller already has the list. Returns the chat's id.
 */
export async function sendNewChat(chat: NewChat, known?: ChatSummary[]): Promise<string> {
  const [only] = chat.members;
  if (chat.kind === 'direct' && only) {
    const chats =
      known ??
      (await Promise.all(
        (await myChats()).map((c) => chatSummary(c.seed, c.at).catch(() => null)),
      ));
    const existing = chats.find(
      (c) => c?.kind === 'direct' && c.members.length === 2 && c.members.includes(only.agent),
    );
    if (existing) {
      await postChatMessage(existing.id, chat.body, null);
      return existing.id;
    }
  }
  const members = chat.members.map((m) => m.agent);
  return startChat(chat.kind, members, chat.title || null, chat.body);
}
