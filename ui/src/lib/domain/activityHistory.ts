import { committedOf, minimumOf, type Activity } from './activities.ts';

/**
 * Past activity as web shows it (web-backend activity_history.py, web's
 * ActivityHistorySection and activityHistoryPresentation.ts): participants check in, and
 * so does the person who asked for a booking (ADR 0077).
 */
export type CompletionTone = 'complete' | 'mixed' | 'uncompleted';

/** web's `_staffing_failed`: fewer people than the roles needed took part. */
export const staffingFailed = (a: Activity): boolean => committedOf(a) < minimumOf(a);

/** One side's check-ins (web's `_completion_side_state` counts). */
export interface Side {
  eligible: number;
  completed: number;
  uncompleted: number;
}

/** A booking's requester side (ADR 0077), or null for other activities. */
export function requesterSide(a: Activity): Side | null {
  if (!a.booking) return null;
  const done = a.completion.requester;
  return { eligible: 1, completed: done === true ? 1 : 0, uncompleted: done === false ? 1 : 0 };
}

/** web's side `statusLabel` / counts line. */
function sideText(c: Side): string {
  if (c.eligible === 0) return 'No eligible people are assigned on this side yet.';
  const pending = Math.max(0, c.eligible - c.completed - c.uncompleted);
  const parts = [`${c.completed}/${c.eligible} completed`, `${c.uncompleted} uncompleted`];
  if (pending > 0) parts.push(`${pending} pending`);
  return parts.join(' · ');
}

/** The participants' line. */
export function completionText(a: Activity): string {
  if (staffingFailed(a)) return 'Marked uncompleted — no participants signed up';
  return sideText(a.completion);
}

/** The requester's line. */
export const requesterText = (side: Side): string => sideText(side);

function states(c: Side): string[] {
  if (c.eligible <= 0) return [];
  const pending = Math.max(0, c.eligible - c.completed - c.uncompleted);
  return [
    ...(c.completed > 0 ? ['completed'] : []),
    ...(c.uncompleted > 0 ? ['uncompleted'] : []),
    ...(pending > 0 ? ['pending'] : []),
  ];
}

/** web's `_aggregate_completion` over the requester and participant sides. */
export function aggregate(a: Activity): { label: string; tone: CompletionTone } {
  const requester = requesterSide(a);
  const all = [
    ...(requester ? states(requester) : []),
    ...(staffingFailed(a) ? ['uncompleted'] : states(a.completion)),
  ];
  if (all.every((s) => s === 'pending')) return { label: 'Completion pending', tone: 'mixed' };
  const done = all.includes('completed');
  const notDone = all.includes('uncompleted');
  if (notDone && !done) return { label: 'Marked uncompleted', tone: 'uncompleted' };
  if (done && !notDone && !all.includes('pending'))
    return { label: 'Marked completed', tone: 'complete' };
  return { label: 'Mixed completion', tone: 'mixed' };
}

/** web-backend `_history_labels`: what kind of past activity this was. */
export function historyContext(a: Activity): { label: string; description: string } {
  if (!a.booking)
    return {
      label: 'Self-planned activity',
      description: 'This activity was scheduled directly on the project calendar.',
    };
  const minimum = minimumOf(a);
  if (committedOf(a) >= minimum && minimum > 0)
    return {
      label: 'Committed activity',
      description: 'Enough participants were assigned before this activity ended.',
    };
  return {
    label: 'Planned activity',
    description: 'This request-linked activity ended before minimum staffing was met.',
  };
}

/** web's `_rating_summary`: the average to one decimal, or null. */
export function ratingAverage(a: Activity): number | null {
  if (a.ratings.length === 0) return null;
  const sum = a.ratings.reduce((n, r) => n + r.stars, 0);
  return Math.round((sum / a.ratings.length) * 10) / 10;
}

/** web's `historyCardTone`. */
export function cardTone(a: Activity): 'good' | 'mixed' | 'poor' {
  const average = ratingAverage(a);
  const tone = aggregate(a).tone;
  if (tone === 'uncompleted' || (average !== null && average < 3)) return 'poor';
  if (tone === 'mixed' || a.ratings.length === 0) return 'mixed';
  return 'good';
}

/** web's `viewerNeedsCompletionFollowUp`: the viewer still owes a check-in. */
export const owesCheckIn = (a: Activity): boolean =>
  (a.completion.requesterCanSet && a.completion.requester === null) ||
  (a.completion.canSet && a.completion.mine === null && !staffingFailed(a));

/** web's follow-up: the viewer still owes a check-in, or may rate. */
export const needsFollowUp = (a: Activity): boolean => owesCheckIn(a) || a.canRate;

/** web's `formatRelativeTime`: "5m ago", "3h ago", "2d ago", then the date. */
export function endedAgo(micros: number, now = Date.now()): string {
  const minutes = Math.round((now - micros / 1000) / 60_000);
  if (minutes < 60) return `${Math.max(minutes, 1)}m ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.round(hours / 24);
  if (days < 7) return `${days}d ago`;
  return new Date(micros / 1000).toLocaleDateString();
}
