/**
 * A meetup's schedule (ADR 0119): `weekly` on a usual day at an optional usual time, or
 * `any-day` (whenever members set a session). The usual day is stored as the first such day,
 * a UTC date and time like any event plan's, and read back as its weekday.
 */

import { locale, t } from '#lib/i18n/index.ts';
import { fromWall, inZone } from './zone.ts';
import type { EventSchedule, ScheduleForm } from './eventSchedule.ts';

const pad = (n: number): string => String(n).padStart(2, '0');

/** The next `weekday` (0 Sunday … 6 Saturday) on or after `from`, as a local `yyyy-mm-dd`. */
export function nextWeekday(weekday: number, from = new Date()): string {
  const d = new Date(from.getFullYear(), from.getMonth(), from.getDate());
  d.setDate(d.getDate() + ((weekday - d.getDay() + 7) % 7));
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

/** Monday first, as people read a week. */
export const WEEKDAYS = [1, 2, 3, 4, 5, 6, 0];

/** "Thursdays" (or the language's plural weekday). */
export function weekdayName(weekday: number): string {
  const day = new Date(2026, 0, 4 + weekday); // 4 Jan 2026 is a Sunday
  return t('{day}s', { day: day.toLocaleDateString(locale(), { weekday: 'long' }) });
}

/** The weekly form as a stored schedule: the first usual day, at the usual time if given. */
export function weeklyFromForm(f: ScheduleForm): EventSchedule {
  const day = nextWeekday(f.weeklyDay);
  const start = fromWall(new Date(`${day}T${f.startTime || '12:00'}`));
  const finish = fromWall(new Date(`${day}T${f.finishTime || f.startTime || '12:00'}`));
  const utcDay = `${start.getUTCFullYear()}-${pad(start.getUTCMonth() + 1)}-${pad(start.getUTCDate())}`;
  const utcTime = (d: Date): string => `${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}`;
  return {
    mode: 'weekly',
    start_date: utcDay,
    end_date: null,
    start_time: f.startTime ? utcTime(start) : null,
    finish_time: f.finishTime ? utcTime(finish) : null,
  };
}

/** "Thursdays · 6:00 pm", "Thursdays", or "Whenever members set a session". */
export function meetupLine(s: EventSchedule): string {
  if (s.mode !== 'weekly' || !s.start_date) return t('Whenever members set a session');
  const at = new Date(`${s.start_date}T${s.start_time ?? '12:00'}:00Z`);
  const day = t('{day}s', {
    day: at.toLocaleDateString(locale(), inZone({ weekday: 'long' })),
  });
  if (!s.start_time) return day;
  return `${day} · ${at.toLocaleTimeString(locale(), inZone({ hour: 'numeric', minute: '2-digit' }))}`;
}

/** The wizard's review line for a meetup form. */
export function meetupSummary(f: ScheduleForm): string {
  if (f.scheduleMode !== 'weekly') return t('Whenever members set a session');
  return f.startTime ? `${weekdayName(f.weeklyDay)} · ${f.startTime}` : weekdayName(f.weeklyDay);
}
