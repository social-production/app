import { mentionsOf, type ChatMessage } from './chat.ts';
import type { Proposal } from './governance.ts';
import type { ProjectMode } from './projects.ts';

/**
 * In-app notifications (web's NotificationItem, ADR 0046). Nothing is stored: each device
 * derives them from the projects opened on its node, the kinds web-backend creates for what
 * the app has so far: phase votes (`prj-phase-vote`), phase changes (`prj-phase-done`), chat
 * replies (`reply`) and @mentions in project, direct and group chats (`mention`, ADR 0047).
 * Times are Holochain micros.
 */
export type NotificationKind = 'prj-phase-vote' | 'prj-phase-done' | 'reply' | 'mention';

/** web's notification settings categories the app has kinds for (web-backend
 * `notification_preferences.py` KIND_TO_CATEGORY); follows, shares, roles and plan-leading
 * come with their features. */
export type NotificationCategory = 'comments' | 'votes_needed' | 'phase_done';

export const categoryOf: Record<NotificationKind, NotificationCategory> = {
  reply: 'comments',
  mention: 'comments',
  'prj-phase-vote': 'votes_needed',
  'prj-phase-done': 'phase_done',
};

export interface NotificationItem {
  id: string;
  kind: NotificationKind;
  /** web's subject: a project, or a direct or group chat (web's "Thread" card). */
  subject: 'project' | 'thread';
  /** The project's or chat's network seed; a project's page is found by seed on opening. */
  seed: string;
  /** Projects only. */
  projectMode: ProjectMode | null;
  actor: string;
  title: string;
  body: string;
  /** Replies: web's "Replied to your comment" / "Replied to your project" lead. */
  lead: string | null;
  /** Where the card goes on the project page (web's href query). */
  target:
    | { tab: 'chat'; comment: string }
    | { open: 'vote'; voteTarget: string }
    | { conversation: string }
    | null;
  createdAt: number;
  isUnread: boolean;
}

/** What one project gives the list. */
export interface ProjectActivity {
  seed: string;
  title: string;
  mode: ProjectMode;
  creator: string;
  amMember: boolean;
  phaseLabel: (id: string) => string;
  proposals: Proposal[];
  messages: ChatMessage[];
}

type Unread = Omit<NotificationItem, 'isUnread'>;

/**
 * Everything one project notifies `me` (whose username is `myName`) of, as web-backend would
 * have sent it.
 */
export function projectNotifications(p: ProjectActivity, me: string, myName: string): Unread[] {
  const base = { subject: 'project' as const, seed: p.seed, projectMode: p.mode };
  const out: Unread[] = [];
  if (p.amMember) {
    for (const v of p.proposals) {
      if (v.author === me) continue;
      // phase_requests.py: the other members get "Vote on advancing to …" when a move opens.
      if ('Phase' in v.kind) {
        out.push({
          ...base,
          id: `vote:${v.id}`,
          kind: 'prj-phase-vote',
          actor: v.author,
          title: p.title,
          body: `Vote on advancing to ${p.phaseLabel(v.kind.Phase.to)}.`,
          lead: null,
          target: { open: 'vote', voteTarget: v.id },
          createdAt: v.createdAt,
        });
      }
      // …and "The project phase changed to …" once a move or a return passes.
      const to = 'Phase' in v.kind ? v.kind.Phase.to : 'Return' in v.kind ? v.kind.Return.to : null;
      if (to !== null && v.status === 'Passed' && v.closedAt !== null) {
        out.push({
          ...base,
          id: `phase:${v.id}`,
          kind: 'prj-phase-done',
          actor: v.author,
          title: 'Project phase change executed',
          body: `The project phase changed to ${p.phaseLabel(to)}.`,
          lead: null,
          target: null,
          createdAt: v.closedAt,
        });
      }
    }
  }
  return [...out, ...chatNotifications(p, me, myName)];
}

/** Chat replies and @mentions (governance.py `_notify_comment_recipients`). */
function chatNotifications(p: ProjectActivity, me: string, myName: string): Unread[] {
  const base = { subject: 'project' as const, seed: p.seed, projectMode: p.mode };
  const out: Unread[] = [];
  const mentionName = myName.toLowerCase();
  for (const m of p.messages) {
    if (m.author === me || (m.kind ?? 'text') !== 'text') continue;
    // governance.py: "@name" pings that person, as well as any reply notice.
    if (mentionName && mentionsOf(m.body).includes(mentionName)) {
      out.push({
        ...base,
        id: `mention:${m.id}`,
        kind: 'mention',
        actor: m.author,
        title: p.title,
        body: 'You were mentioned.',
        lead: null,
        target: { tab: 'chat', comment: m.id },
        createdAt: m.createdAt,
      });
    }
    // governance.py `_notify_comment_recipients`: the replied-to author, else the owner.
    const lead =
      m.replyTo?.author === me
        ? 'Replied to your comment'
        : p.creator === me
          ? 'Replied to your project'
          : null;
    if (!lead) continue;
    out.push({
      ...base,
      id: `reply:${m.id}`,
      kind: 'reply',
      actor: m.author,
      title: p.title,
      body: m.body.trim(),
      lead,
      target: { tab: 'chat', comment: m.id },
      createdAt: m.createdAt,
    });
  }
  return out;
}

/** What one direct or group chat gives the list. */
export interface ChatActivity {
  seed: string;
  /** A group's name; `null` for a direct chat or an unnamed group. */
  title: string | null;
  messages: ChatMessage[];
  /** Display names of the messages' authors. */
  names: Record<string, string>;
}

/** messaging.py `_notify_chat_mentions`: "@name" in a chat you're in pings you. */
export function chatMentionNotifications(c: ChatActivity, me: string, myName: string): Unread[] {
  const name = myName.toLowerCase();
  const where = c.title ? `in ${c.title}` : 'in a chat';
  return c.messages
    .filter(
      (m) =>
        m.author !== me &&
        (m.kind ?? 'text') === 'text' &&
        !!name &&
        mentionsOf(m.body).includes(name),
    )
    .map((m) => ({
      id: `mention:${m.id}`,
      kind: 'mention' as const,
      subject: 'thread' as const,
      seed: c.seed,
      projectMode: null,
      actor: m.author,
      title: c.title ?? 'Chat',
      body: `${c.names[m.author] ?? 'Someone'} mentioned you ${where}.`,
      lead: null,
      target: { conversation: c.seed },
      createdAt: m.createdAt,
    }));
}

/** What this device has read: everything up to `all`, plus single cards read since. */
export interface ReadMarks {
  all: number;
  ids: string[];
}

/** Newest first, each marked read or unread. */
export function withReadMarks(items: Unread[], marks: ReadMarks): NotificationItem[] {
  return items
    .map((n) => ({ ...n, isUnread: n.createdAt > marks.all && !marks.ids.includes(n.id) }))
    .sort((a, b) => b.createdAt - a.createdAt);
}

/** Read one card; ids at or before `all` are dropped, so the list stays short. */
export function markRead(marks: ReadMarks, item: { id: string }, items: Unread[]): ReadMarks {
  const after = new Set(items.filter((n) => n.createdAt > marks.all).map((n) => n.id));
  return {
    all: marks.all,
    ids: [...new Set([...marks.ids, item.id])].filter((id) => after.has(id)),
  };
}
