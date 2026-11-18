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

/** activities.py: "<activity> was added." to the project's other members. */
export function activityAddedNotice(
  members: string[],
  project: string,
  title: string,
  activity: { id: string; title: string },
): Promise<void> {
  return notify(
    members,
    'prj-activity',
    () => title,
    `${activity.title} was added.`,
    activityLink(project, activity.id),
  );
}

/** service_requests.py: a new request, or a decision on one ("Request accepted: …"). */
export function requestNotice(
  to: string[],
  request: { project: string; id: string; service: string; title: string },
  status: string,
): Promise<void> {
  return notify(
    to,
    'svc-request',
    () => request.service,
    `Request ${status}: ${request.title}`,
    `/projects/${request.project}?request=${encodeURIComponent(request.id)}`,
  );
}

/** phase_requests.py: "The project phase changed to …" to the other members. */
export function phaseDoneNotice(members: string[], project: string, to: string): Promise<void> {
  return notify(
    members,
    'prj-phase-done',
    () => 'Project phase change executed',
    `The project phase changed to ${to}.`,
    `/projects/${project}`,
  );
}

/** projects_plans.py / events_plans.py: to the new leading plan's author. */
export function planLeadNotice(author: string, project: string, event: boolean): Promise<void> {
  return notify(
    [author],
    event ? 'evt-plan-lead' : 'prj-plan-lead',
    () => 'Plan became leading',
    event
      ? 'A vote passed and your event plan is now leading.'
      : 'A vote passed and your plan is now leading.',
    `/projects/${project}`,
  );
}

/** software/pull_requests.py: to the pull request's author once its vote passes. */
export function prApprovedNotice(author: string, project: string, pr: string): Promise<void> {
  return notify(
    [author],
    'pr-approved',
    () => 'Pull request approved',
    'Voting passed and your pull request is approved for merge.',
    `/projects/${project}?open=vote&voteTarget=${encodeURIComponent(pr)}&voteKind=pull_request_merge`,
  );
}

/** bot_reviews.py: a review opened or closed, to the account and to whoever marked it. */
export function botReviewNotice(
  to: string[],
  kind: 'bot-review' | 'bot-review-result',
  copy: [string, string],
  account: string,
): Promise<void> {
  return notify(to, kind, () => copy[0], copy[1], `/profile/${account}`);
}

/** moderation/effects.py: a report removed the project; its members hear. */
export function removedNotice(members: string[], title: string, reason: string): Promise<void> {
  return notify(
    members,
    'prj-removed',
    () => title,
    `This project was removed after a ${reason} report.`,
    '/',
  );
}
