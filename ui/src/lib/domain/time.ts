import { locale, t } from '#lib/i18n/index.ts';

/** Time labels as web shows them (web/src/lib/utils/time.ts). Times are Holochain micros. */

const MINUTE_MS = 60_000;

/** "5m", "3h", "2d", then the date ("Oct 12"). */
export function relativeCompact(micros: number, nowMs = Date.now()): string {
  const minutes = Math.max(Math.round(Math.abs(nowMs - micros / 1000) / MINUTE_MS), 1);
  if (minutes < 60) return t('{n}m', { n: minutes });
  const hours = Math.round(minutes / 60);
  if (hours < 24) return t('{n}h', { n: hours });
  const days = Math.round(hours / 24);
  if (days < 7) return t('{n}d', { n: days });
  return new Date(micros / 1000).toLocaleDateString(locale(), { month: 'short', day: 'numeric' });
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
  const when = relativeCompact(activityInstant(createdAt, updatedAt), nowMs);
  return updated ? t('updated {when}', { when }) : t('created {when}', { when });
}

/** web's `formatRelativeTime`: "5m ago", "3h ago", "2d ago", then the date. */
export function relativeTime(micros: number, nowMs = Date.now()): string {
  const minutes = Math.round((nowMs - micros / 1000) / MINUTE_MS);
  if (minutes < 60) return t('{n}m ago', { n: Math.max(minutes, 1) });
  const hours = Math.round(minutes / 60);
  if (hours < 24) return t('{n}h ago', { n: hours });
  const days = Math.round(hours / 24);
  if (days < 7) return t('{n}d ago', { n: days });
  return new Date(micros / 1000).toLocaleDateString(locale());
}

/** When an opened post or thread was written: "3:12 pm · 13 Nov 2026" in the reader's locale. */
export function postedAt(micros: number): string {
  const at = new Date(micros / 1000);
  const time = at.toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' });
  const day = at.toLocaleDateString(undefined, { day: 'numeric', month: 'short', year: 'numeric' });
  return `${time} · ${day}`;
}

/** The calendar day after a local "YYYY-MM-DD" (noon, so a clock change can't skip a day). */
export function nextDay(day: string): string {
  const d = new Date(`${day}T12:00`);
  d.setDate(d.getDate() + 1);
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}
