import type { ChatMessage } from '#lib/domain/chat.ts';
import type {
  ChatKind,
  ChatPin,
  ChatSummary,
  Contact,
  ListPrefs,
  NewChat,
} from '#lib/domain/messages.ts';
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
  manager: AgentPubKey | null;
  members: AgentPubKey[];
  title: string | null;
  last: {
    author: AgentPubKey;
    preview: string;
    created_at: number;
    kind: 'text' | 'system';
  } | null;
  unread: number;
  member: boolean;
  can_pin: boolean;
  pinned: boolean;
  muted: boolean;
  hidden: boolean;
}

interface PinWire {
  message: ActionHash;
  preview: string;
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
    manager: s.manager && hashToString(s.manager),
    members: s.members.map(hashToString),
    title: s.title,
    last: s.last && {
      author: hashToString(s.last.author),
      preview: s.last.preview,
      createdAt: s.last.created_at,
      kind: s.last.kind,
    },
    unread: s.unread,
    invitedAt,
    member: s.member,
    canPin: s.can_pin,
    pinned: s.pinned,
    muted: s.muted,
    hidden: s.hidden,
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

/** web's `edit_message`: a new text for your own message. */
export async function editChatMessage(seed: string, id: string, body: string): Promise<void> {
  await chat<null>(seed, 'edit_message', { id: stringToHash(id), body });
}

/** web's pin_message / unpin_message (at most 5). */
export async function pinChatMessage(seed: string, id: string, pinned: boolean): Promise<void> {
  await chat<null>(seed, 'pin_message', { id: stringToHash(id), pinned });
}

/** The pinned messages, oldest pin first. */
export async function chatPins(seed: string): Promise<ChatPin[]> {
  const wire = await chat<PinWire[]>(seed, 'pins');
  return wire.map((p) => ({ messageId: hashToString(p.message), preview: p.preview }));
}

/** web's set_conversation_preferences: your own pin, mute and delete (hide). */
export async function setChatPrefs(seed: string, prefs: ListPrefs): Promise<void> {
  await chat<null>(seed, 'set_prefs', {
    pinned: prefs.pinned ?? null,
    muted: prefs.muted ?? null,
    hidden: prefs.hidden ?? null,
  });
}

/** web's rename_group_conversation (the manager only). */
export async function renameGroup(seed: string, title: string): Promise<void> {
  await chat<null>(seed, 'rename_group', title);
}

/** web's add_group_member: invite them in the chat, then send them its seed. */
export async function addGroupMember(seed: string, agent: string): Promise<void> {
  await chat<null>(seed, 'add_member', stringToHash(agent));
  await inbox<null>('send_invite', { to: stringToHash(agent), seed });
}

export async function removeGroupMember(seed: string, agent: string): Promise<void> {
  await chat<null>(seed, 'remove_member', stringToHash(agent));
}

/** web's leave_group: a leaving manager hands the group to the longest-standing member. */
export async function leaveGroup(seed: string): Promise<void> {
  await chat<null>(seed, 'leave');
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
