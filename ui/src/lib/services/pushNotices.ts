import type { NotificationKind } from '#lib/domain/notifications.ts';

import { notify } from './notices.ts';
import { myAgent } from './people.ts';

/**
 * Push for the cards a write makes for someone else (ADR 0108), worded as those cards are
 * (domain/*Notifications.ts). Each is a best-effort nudge: the card is derived on their
 * device either way. Chat replies and @mentions are in `notices.ts`.
 */

const me = (): Promise<string> => myAgent().catch(() => '');

/** An in-app path from a link that may be absolute. */
function pathOf(url: string): string {
  try {
    const u = new URL(url, 'https://app.invalid');
    return `${u.pathname}${u.search}`;
  } catch {
    return '/';
  }
}

export async function followNotice(target: string, pending: boolean): Promise<void> {
  await notify(
    [target],
    pending ? 'follow-request' : 'new-follower',
    (name) => (pending ? `${name} requested to follow you` : `${name} started following you`),
    '',
    `/profile/${await me()}`,
  );
}

export async function followAcceptedNotice(follower: string): Promise<void> {
  await notify(
    [follower],
    'follow-accepted',
    (name) => `${name} accepted your follow request`,
    '',
    `/profile/${await me()}`,
  );
}

const SHARE_KIND: Record<string, NotificationKind> = {
  project: 'prj-share',
  event: 'evt-share',
  'help request': 'hr-share',
  thread: 'thread-share',
  post: 'post-share',
};

/** shares.py: "<name> shared a project with you", the item's title under it. */
export function shareNotice(to: string, label: string, title: string, url: string): Promise<void> {
  const kind = SHARE_KIND[label] ?? 'prj-share';
  return notify([to], kind, (name) => `${name} shared a ${label} with you`, title, pathOf(url));
}

export function inviteNotice(to: string, community: boolean): Promise<void> {
  return notify(
    [to],
    community ? 'community-invite' : 'evt-invite',
    (name) => `${name} invited you to ${community ? 'a community' : 'an event'}`,
    'Accept or decline it in Invites.',
    '/invites',
  );
}

const activityLink = (project: string, activity: string): string =>
  `/projects/${project}?activity=${encodeURIComponent(activity)}`;

/** To the activity's author: '<name> took the "<role>" role', the activity under it. */
export function roleTakenNotice(
  author: string,
  project: string,
  activity: { id: string; title: string },
  role: string,
): Promise<void> {
  return notify(
    [author],
    'prj-role-taken',
    (name) => `${name} took the "${role}" role`,
    activity.title,
    activityLink(project, activity.id),
  );
}

export function roleSuggestedNotice(
  person: string,
  project: string,
  activity: { id: string; title: string },
  role: string,
): Promise<void> {
  return notify(
    [person],
    'prj-role-suggest',
    () => `Suggested for ${role}`,
    `You've been suggested for ${role} on ${activity.title}.`,
    activityLink(project, activity.id),
  );
}

/** phase_requests.py: the other members are asked to vote on the move. */
export function phaseVoteNotice(
  members: string[],
  project: string,
  title: string,
  to: string,
): Promise<void> {
  return notify(
    members,
    'prj-phase-vote',
    () => title,
    `Vote on advancing to ${to}.`,
    `/projects/${project}`,
  );
}
