import { signupTone, type Activity } from './activities.ts';
import { localNow } from './activityForm.ts';

/**
 * web's ProjectActivityCalendarCard month grid: 6 weeks from Monday, each day with its
 * activities as bars on a 24-hour track, overlapping ones side by side. Browser time (web uses
 * its Display timezone setting, which the app doesn't have yet).
 */
export interface DayBar {
  id: string;
  title: string;
  tone: 'red' | 'yellow' | 'green' | 'muted';
  label: string;
  top: number;
  height: number;
  left: number;
  width: number;
}

export interface CalendarDay {
  iso: string;
  day: number;
  inMonth: boolean;
  bars: DayBar[];
}

const MINUTES = 24 * 60;
const TONE = { empty: 'red', partial: 'yellow', met: 'green' } as const;

export function isoOf(date: Date): string {
  return localNow(date).slice(0, 10);
}

const time = (d: Date): string => d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

function bars(iso: string, activities: Activity[], now: number): DayBar[] {
  const dayStart = new Date(`${iso}T00:00`);
  const dayEnd = new Date(dayStart);
  dayEnd.setDate(dayEnd.getDate() + 1);
  const ranged = activities
    .filter((a) => a.startsAt / 1000 < dayEnd.getTime() && a.endsAt / 1000 > dayStart.getTime())
    .map((a) => {
      const from = new Date(Math.max(a.startsAt / 1000, dayStart.getTime()));
      const to = new Date(Math.min(a.endsAt / 1000, dayEnd.getTime()));
      const start = (from.getTime() - dayStart.getTime()) / 60_000;
      const end = Math.max((to.getTime() - dayStart.getTime()) / 60_000, start + 30);
      const ended = !a.rolesOpen || a.endsAt / 1000 <= now;
      const tone: DayBar['tone'] = ended ? 'muted' : TONE[signupTone(a)];
      return { a, start, end, tone, label: `${time(from)} to ${time(to)}`, col: 0 };
    })
    .sort((x, y) => x.start - y.start);
  const colEnds: number[] = [];
  for (const item of ranged) {
    const free = colEnds.findIndex((end) => end <= item.start);
    item.col = free === -1 ? colEnds.length : free;
    colEnds[item.col] = item.end;
  }
  return ranged.map((item) => {
    const overlap = Math.max(
      1,
      ranged.filter((o) => item.start < o.end && o.start < item.end).length,
    );
    return {
      id: item.a.id,
      title: item.a.title,
      tone: item.tone,
      label: item.label,
      top: (item.start / MINUTES) * 100,
      height: ((item.end - item.start) / MINUTES) * 100,
      left: (item.col / overlap) * 100,
      width: 100 / overlap,
    };
  });
}

export function monthDays(month: Date, activities: Activity[], now = Date.now()): CalendarDay[] {
  const start = new Date(month);
  start.setDate(month.getDate() - ((month.getDay() + 6) % 7));
  return Array.from({ length: 42 }, (_, i) => {
    const date = new Date(start);
    date.setDate(start.getDate() + i);
    const iso = isoOf(date);
    return {
      iso,
      day: date.getDate(),
      inMonth: date.getMonth() === month.getMonth(),
      bars: bars(iso, activities, now),
    };
  });
}

export const monthStart = (date: Date): Date => new Date(date.getFullYear(), date.getMonth(), 1);

/** web's `defaultVisibleMonthStart`: the next upcoming activity's month, else the first's. */
export function defaultMonth(activities: Activity[], now = Date.now()): Date {
  const starts = activities.map((a) => a.startsAt / 1000).sort((x, y) => x - y);
  const next = starts.find((t) => t >= now) ?? starts[0];
  return monthStart(next === undefined ? new Date(now) : new Date(next));
}

/** web's `setDefaultActivityTimes`: a picked day's first activity, else 18:00–19:00 that day
 * (the next hour if that has passed), else the next hour. */
export function defaultTimes(
  iso: string | null,
  activities: Activity[],
  now = new Date(),
): { scheduledAt: string; endsAt: string } {
  const nextHour = (): { scheduledAt: string; endsAt: string } => {
    const s = new Date(now);
    s.setMinutes(0, 0, 0);
    s.setHours(s.getHours() + 1);
    return { scheduledAt: localNow(s), endsAt: localNow(new Date(s.getTime() + 3_600_000)) };
  };
  if (!iso) return nextHour();
  const first = activities
    .filter((a) => isoOf(new Date(a.startsAt / 1000)) === iso)
    .sort((x, y) => x.startsAt - y.startsAt)[0];
  if (first) {
    return {
      scheduledAt: localNow(new Date(first.startsAt / 1000)),
      endsAt: localNow(new Date(first.endsAt / 1000)),
    };
  }
  if (new Date(`${iso}T18:00`).getTime() < now.getTime()) return nextHour();
  return { scheduledAt: `${iso}T18:00`, endsAt: `${iso}T19:00` };
}
