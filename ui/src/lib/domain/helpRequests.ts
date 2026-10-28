/** Help requests (ADR 0067): commons spaces with mode `help-request` and one activity. */

import { zoneOf } from './activityTime.ts';

export const HELP_MODE = 'help-request';

export const isHelpRequest = (mode: string | undefined): boolean => mode === HELP_MODE;

/** web's RoleDraft: a role's title, details and slots (0 = any number). */
export interface HelpRoleDraft {
  label: string;
  description: string;
  slots: number;
}

/** web's slots as an activity role's counts (sp_help::role_counts). */
export function roleCounts(slots: number): { required: number; maximum: number | null } {
  const n = Math.max(0, Math.floor(slots) || 0);
  return n === 0 ? { required: 1, maximum: null } : { required: n, maximum: n };
}

/** An activity role's slots as web shows them (0 = any number). */
export const slotsOf = (maximum: number | null): number => maximum ?? 0;

/** web's `signupSummary` on the help-request card. */
export function signupSummary(signups: number | null, needed: number | null): string {
  if (signups !== null && needed !== null && needed > 0)
    return `${signups} signed up · ${needed} needed`;
  return signups ? `${signups} signed up` : '';
}

/** `datetime-local` value for a date, in the browser's zone (web's `toLocalDateTimeInput`). */
export function toLocalInput(date: Date): string {
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

/** web's defaultNeededAt (`null`: now, on the hour) and defaultEndsAt: `hours` later. */
export function hoursAfter(value: string | null, hours: number): string {
  const at = value === null ? new Date() : new Date(value);
  if (Number.isNaN(at.getTime())) return '';
  if (value === null) at.setMinutes(0, 0, 0);
  at.setHours(at.getHours() + hours);
  return toLocalInput(at);
}

/** A `datetime-local` value as Holochain microseconds. */
export const inputMicros = (value: string): number => new Date(value).getTime() * 1000;

export const endsAfter = (start: string, end: string): boolean =>
  new Date(end).getTime() > new Date(start).getTime();

/** web's `formatScheduleRange` (create overview): start, and the end's time or date. */
export function scheduleLabel(startValue: string, endValue: string): string {
  const start = startValue ? new Date(startValue) : null;
  const end = endValue ? new Date(endValue) : null;
  if (!start || Number.isNaN(start.getTime())) return 'Time TBD';
  const startLabel = start.toLocaleString(undefined, {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  });
  if (!end || Number.isNaN(end.getTime()) || end <= start) return startLabel;
  if (end.toDateString() === start.toDateString())
    return `${startLabel} – ${end.toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' })}`;
  return `${startLabel} – ${end.toLocaleString(undefined, {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  })}`;
}

/** web's `formatLocalDateTime` (help-request card and header): "Fri, Nov 6, 2026, 9:00 AM AEDT". */
export function localDateTime(micros: number | null): string {
  if (micros === null) return '';
  const date = new Date(micros / 1000);
  const formatted = date.toLocaleString(undefined, {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  });
  return `${formatted} ${zoneOf(date)}`.trim();
}
