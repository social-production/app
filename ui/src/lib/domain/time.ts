/** Time labels as web shows them (web/src/lib/utils/time.ts). Times are Holochain micros. */

const MINUTE_MS = 60_000;

/** "5m", "3h", "2d", then the date ("Oct 12"). */
export function relativeCompact(micros: number, nowMs = Date.now()): string {
  const minutes = Math.max(Math.round(Math.abs(nowMs - micros / 1000) / MINUTE_MS), 1);
  if (minutes < 60) return `${minutes}m`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours}h`;
  const days = Math.round(hours / 24);
  if (days < 7) return `${days}d`;
  return new Date(micros / 1000).toLocaleDateString(undefined, { month: 'short', day: 'numeric' });
}

/** The instant a clock stamp shows: the update when it is newer than creation. */
export function activityInstant(createdAt: number, updatedAt: number | null): number {
  return updatedAt !== null && updatedAt > createdAt ? updatedAt : createdAt;
}

/** "created 3h" / "updated 5m", for tooltips. */
export function describeUpdate(
  createdAt: number,
  updatedAt: number | null,
  nowMs = Date.now(),
): string {
  const updated = updatedAt !== null && updatedAt > createdAt;
  return `${updated ? 'updated' : 'created'} ${relativeCompact(activityInstant(createdAt, updatedAt), nowMs)}`;
}

/** web's `formatRelativeTime`: "5m ago", "3h ago", "2d ago", then the date. */
export function relativeTime(micros: number, nowMs = Date.now()): string {
  const minutes = Math.round((nowMs - micros / 1000) / MINUTE_MS);
  if (minutes < 60) return `${Math.max(minutes, 1)}m ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.round(hours / 24);
  if (days < 7) return `${days}d ago`;
  return new Date(micros / 1000).toLocaleDateString();
}

/** When an opened post or thread was written: "3:12 pm · 13 Nov 2026" in the reader's locale. */
export function postedAt(micros: number): string {
  const at = new Date(micros / 1000);
  const time = at.toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' });
  const day = at.toLocaleDateString(undefined, { day: 'numeric', month: 'short', year: 'numeric' });
  return `${time} · ${day}`;
}
