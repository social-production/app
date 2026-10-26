import { committedOf, minimumOf, type Activity } from './activities.ts';

/**
 * Past activity as web shows it (web-backend activity_history.py, web's
 * ActivityHistorySection and activityHistoryPresentation.ts), for project activities that
 * weren't booked through a service request: only the participants' side checks in.
 */
export type CompletionTone = 'complete' | 'mixed' | 'uncompleted';

/** web's `_staffing_failed`: fewer people than the roles needed took part. */
export const staffingFailed = (a: Activity): boolean => committedOf(a) < minimumOf(a);

/** web's participant `statusLabel` / counts line. */
export function completionText(a: Activity): string {
  if (staffingFailed(a)) return 'Marked uncompleted — no participants signed up';
  const c = a.completion;
  if (c.eligible === 0) return 'No eligible people are assigned on this side yet.';
  const pending = Math.max(0, c.eligible - c.completed - c.uncompleted);
  const parts = [`${c.completed}/${c.eligible} completed`, `${c.uncompleted} uncompleted`];
  if (pending > 0) parts.push(`${pending} pending`);
  return parts.join(' · ');
}

/** web's `_aggregate_completion`: its label and tone. */
export function aggregate(a: Activity): { label: string; tone: CompletionTone } {
  if (staffingFailed(a)) return { label: 'Marked uncompleted', tone: 'uncompleted' };
  const c = a.completion;
  const pending = Math.max(0, c.eligible - c.completed - c.uncompleted);
  const states = [c.completed > 0, c.uncompleted > 0, pending > 0];
  const [done, notDone, waiting] = states;
  if (c.eligible === 0 || (!done && !notDone))
    return { label: 'Completion pending', tone: 'mixed' };
  if (notDone && !done) return { label: 'Marked uncompleted', tone: 'uncompleted' };
  if (done && !notDone && !waiting) return { label: 'Marked completed', tone: 'complete' };
  return { label: 'Mixed completion', tone: 'mixed' };
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

/** web's follow-up: the viewer still owes a check-in, or may rate. */
export const needsFollowUp = (a: Activity): boolean =>
  (a.completion.canSet && a.completion.mine === null && !staffingFailed(a)) || a.canRate;

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
