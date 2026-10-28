import type { ChatMessage } from '#lib/domain/chat.ts';
import type { ListPrefs, PublicChat } from '#lib/domain/messages.ts';
import type { Person } from '#lib/domain/people.ts';
import { callCell, hashToString, projectCell, type AgentPubKey } from '#lib/holo/index.ts';

import { chatReadAt, listMessages, markChatRead } from './chat.ts';
import { getPerson, myAgent } from './people.ts';
import { myProjectSeeds } from './projects.ts';

/** The part of `get_project` the list shows. */
interface GlanceWire {
  charter: { title: string } | null;
  members: AgentPubKey[];
  am_member: boolean;
  created_at: number | null;
  last_activity_at: number | null;
}

/** web's preview length (linked_chats.py `_comment_preview`). */
const PREVIEW_MAX = 200;

/**
 * web's Public tab (linked_chats.py `get_linked_chats`, projects only): the chats of projects
 * you're a member of (and private events opened on this node; ADR 0044, 0065).
 */
export async function myProjectChats(): Promise<PublicChat[]> {
  const seeds = await myProjectSeeds();
  if (seeds.length === 0) return [];
  const me = await myAgent();
  const settled = await Promise.allSettled(seeds.map((seed) => projectChat(seed, me)));
  return settled.flatMap((r) => (r.status === 'fulfilled' && r.value ? [r.value] : []));
}

async function projectChat(seed: string, me: string): Promise<PublicChat | null> {
  const cell = await projectCell(seed, 'Project');
  const [project, messages, readAt] = await Promise.all([
    callCell<GlanceWire | null>(cell, 'project', 'get_project'),
    listMessages({ seed }),
    chatReadAt({ seed }),
  ]);
  if (!project?.charter) return null;
  if (!project.am_member && !messages.some((m) => m.author === me)) return null;
  const last = messages.at(-1);
  const prefs = prefsOf(me, seed);
  if (prefs.hiddenAt !== null && (last?.createdAt ?? 0) <= prefs.hiddenAt) return null;
  return {
    id: seed,
    title: project.charter.title,
    members: project.members.length,
    last: last
      ? { author: last.author, preview: last.body.slice(0, PREVIEW_MAX), createdAt: last.createdAt }
      : null,
    unread: unreadAfter(messages, readAt ?? 0, me),
    activeAt: last?.createdAt ?? project.last_activity_at ?? project.created_at ?? 0,
    pinned: prefs.pinned,
    muted: prefs.muted,
  };
}

/** web's members panel from a project chat's header (MessagesPage `openProjectMembers`): the
 * project's members as they stand now. */
export async function projectChatMembers(seed: string): Promise<Person[]> {
  const cell = await projectCell(seed, 'Project');
  const project = await callCell<GlanceWire | null>(cell, 'project', 'get_project');
  return Promise.all((project?.members ?? []).map((m) => getPerson(hashToString(m))));
}

function unreadAfter(messages: ChatMessage[], since: number, me: string): number {
  return messages.filter((m) => m.author !== me && m.createdAt > since).length;
}

/**
 * Mark a project's chat (by its seed) read up to `at`, its newest message; never backwards.
 * The mark is a private entry on your chain (ADR 0056), so every device you use agrees.
 */
export async function markProjectChatRead(seed: string, at: number): Promise<void> {
  await markChatRead({ seed }, at);
}

interface StoredPrefs {
  pinned: boolean;
  muted: boolean;
  /** Hidden until a message newer than this (Holochain micros) arrives. */
  hiddenAt: number | null;
}

/** web's chat menu on a project chat (subject_chat_reads' pin/mute/hide), kept in the browser
 * (ADR 0044, 0045); read marks moved to the chain with ADR 0056. */
const prefsKey = (me: string, seed: string): string => `sp.projectChatPrefs.${me}.${seed}`;

function prefsOf(me: string, seed: string): StoredPrefs {
  const none: StoredPrefs = { pinned: false, muted: false, hiddenAt: null };
  try {
    const raw = localStorage.getItem(prefsKey(me, seed));
    return raw ? { ...none, ...(JSON.parse(raw) as Partial<StoredPrefs>) } : none;
  } catch {
    return none;
  }
}

/** Pin, mute or delete (hide until something new arrives) a project chat on this device. */
export async function setProjectChatPrefs(
  seed: string,
  prefs: ListPrefs,
  newestAt: number,
): Promise<void> {
  const me = await myAgent();
  const next = prefsOf(me, seed);
  if (prefs.pinned !== undefined) next.pinned = prefs.pinned;
  if (prefs.muted !== undefined) next.muted = prefs.muted;
  if (prefs.hidden !== undefined) next.hiddenAt = prefs.hidden ? newestAt : null;
  try {
    localStorage.setItem(prefsKey(me, seed), JSON.stringify(next));
  } catch {
    // Private windows may refuse storage; the setting just doesn't stick.
  }
}
