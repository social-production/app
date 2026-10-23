import { mentionsOf, type ChatMessage } from './chat.ts';
import type { Proposal } from './governance.ts';
import type { ProjectMode } from './projects.ts';

/**
 * In-app notifications (web's NotificationItem, ADR 0046). Nothing is stored: each device
 * derives them from the projects opened on its node, the kinds web-backend creates for what
 * the app has so far: phase votes (`prj-phase-vote`), phase changes (`prj-phase-done`), chat
 * replies (`reply`) and @mentions in project chats (`mention`, ADR 0047). Times are Holochain
 * micros.
 */
export type NotificationKind = 'prj-phase-vote' | 'prj-phase-done' | 'reply' | 'mention';

export interface NotificationItem {
  id: string;
  kind: NotificationKind;
  /** The project's network seed; its page is found by seed when the card opens. */
  seed: string;
  projectMode: ProjectMode;
  actor: string;
  title: string;
  body: string;
  /** Replies: web's "Replied to your comment" / "Replied to your project" lead. */
  lead: string | null;
  /** Where the card goes on the project page (web's href query). */
  target: { tab: 'chat'; comment: string } | { open: 'vote'; voteTarget: string } | null;
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
  const base = { seed: p.seed, projectMode: p.mode };
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
  const base = { seed: p.seed, projectMode: p.mode };
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
