import {
  markRead,
  chatMentionNotifications,
  projectNotifications,
  withReadMarks,
  type NotificationItem,
  type ReadMarks,
} from '#lib/domain/notifications.ts';
import type { ProjectMode } from '#lib/domain/projects.ts';
import {
  callCell,
  clonedSeeds,
  hashToString,
  projectCell,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { listMessages } from './chat.ts';
import { chatSummary, listChatMessages, myChats } from './messages.ts';
import { listProposals } from './governance.ts';
import { myAgent, namesOf, viewer } from './people.ts';
import { listingIdOf } from './projects.ts';

/** The part of `get_project` notifications read. */
interface GlanceWire {
  creator: AgentPubKey;
  charter: { title: string; mode: ProjectMode } | null;
  am_member: boolean;
  phases: { id: string; label: string }[];
}

/**
 * web's notifications list, derived on this device (ADR 0046) from the projects opened on its
 * node, as the Public tab is (ADR 0044): reading any other project would join its network.
 */
export async function myNotifications(): Promise<{ items: NotificationItem[] }> {
  const [{ agent: me, name }, seeds, chats] = await Promise.all([
    viewer(),
    clonedSeeds('project'),
    myChats(),
  ]);
  const settled = await Promise.allSettled([
    ...seeds.map((seed) => fromProject(seed, me, name)),
    ...chats.map((c) => fromChat(c.seed, c.at, me, name)),
  ]);
  const all = settled.flatMap((r) => (r.status === 'fulfilled' ? r.value : []));
  return { items: withReadMarks(all, marksOf(me)) };
}

async function fromProject(seed: string, me: string, myName: string) {
  const cell = await projectCell(seed, 'Project');
  const [project, proposals, messages] = await Promise.all([
    callCell<GlanceWire | null>(cell, 'project', 'get_project'),
    listProposals({ seed }),
    listMessages({ seed }),
  ]);
  if (!project?.charter) return [];
  const labels = new Map(project.phases.map((p) => [p.id, p.label]));
  return projectNotifications(
    {
      seed,
      title: project.charter.title,
      mode: project.charter.mode,
      creator: hashToString(project.creator),
      amMember: project.am_member,
      phaseLabel: (id) => labels.get(id) ?? id,
      proposals,
      messages,
    },
    me,
    myName,
  );
}

/** Mentions in a direct or group chat you're still in (web notifies only its members). */
async function fromChat(seed: string, at: number, me: string, myName: string) {
  const [summary, messages] = await Promise.all([chatSummary(seed, at), listChatMessages(seed)]);
  if (!summary.member) return [];
  const found = chatMentionNotifications(
    { seed, title: summary.title, messages, names: {} },
    me,
    myName,
  );
  if (found.length === 0) return [];
  const names = await namesOf([...new Set(found.map((n) => n.actor))]);
  return chatMentionNotifications({ seed, title: summary.title, messages, names }, me, myName);
}

/** Read one card (web's "Mark read"). */
export async function markNotificationRead(item: NotificationItem, all: NotificationItem[]) {
  const me = await myAgent();
  saveMarks(me, markRead(marksOf(me), item, all));
}

/** web's "Mark all read": everything up to the newest card. */
export async function markAllNotificationsRead(all: NotificationItem[]): Promise<void> {
  const me = await myAgent();
  const marks = marksOf(me);
  const newest = Math.max(marks.all, ...all.map((n) => n.createdAt));
  // Pressed before the list loaded, it must not forget the cards already read one by one.
  saveMarks(me, { all: newest, ids: newest === marks.all ? marks.ids : [] });
}

/**
 * Read marks live in the browser per person, like the Public tab's (ADR 0044). The first look
 * on a device counts everything before it as read: web only notified the people who were
 * members when something happened, which a device can't tell for the past.
 */
const marksKey = (me: string): string => `sp.notificationsRead.${me}`;

function marksOf(me: string): ReadMarks {
  try {
    const raw = localStorage.getItem(marksKey(me));
    if (raw) return { all: 0, ids: [], ...(JSON.parse(raw) as Partial<ReadMarks>) };
    const first: ReadMarks = { all: Date.now() * 1000, ids: [] };
    saveMarks(me, first);
    return first;
  } catch {
    return { all: 0, ids: [] };
  }
}

function saveMarks(me: string, marks: ReadMarks): void {
  try {
    localStorage.setItem(marksKey(me), JSON.stringify(marks));
  } catch {
    // Private windows may refuse storage; cards just stay unread here.
  }
}

/** Project pages by seed, looked up once per page view. */
const pages = new Map<string, Promise<string | null>>();

/** The listing id of a card's project, for its link. */
export function projectPageOf(seed: string): Promise<string | null> {
  let found = pages.get(seed);
  if (!found) {
    found = listingIdOf(seed);
    pages.set(seed, found);
    found.then(
      (id) => id ?? pages.delete(seed),
      () => pages.delete(seed),
    );
  }
  return found;
}
