/**
 * The activity wizard's day and times (Activity creation mockup v2, owner 2026-11-25: "a date
 * picker and scroll thing but still simple"): the form keeps `datetime-local` stamps; the
 * calendar sets their day and the wheels their times.
 */
import type { EventSchedule } from './eventSchedule.ts';

const pad = (n: number): string => String(n).padStart(2, '0');

export const dayOf = (stamp: string): string => stamp.slice(0, 10);
export const timeOf = (stamp: string): string => stamp.slice(11, 16);
export const stampOf = (day: string, time: string): string => `${day}T${time}`;

export function localDayOf(d: Date): string {
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

/** Minutes after midnight of an `HH:MM`. */
const minutes = (time: string): number => {
  const [h = '0', m = '0'] = time.split(':');
  return Number(h) * 60 + Number(m);
};
const clock = (total: number): string => {
  const t = ((total % 1440) + 1440) % 1440;
  return `${pad(Math.floor(t / 60))}:${pad(t % 60)}`;
};

/** Every time of day in `step`-minute steps, as `HH:MM`. */
export function timeOptions(step = 5): string[] {
  return Array.from({ length: 1440 / step }, (_, i) => clock(i * step));
}

/** A time rounded to the wheel's step. */
export const snapTime = (time: string, step = 5): string =>
  clock(Math.round(minutes(time) / step) * step);

type Times = { scheduledAt: string; endsAt: string };

/** The same times on another day. */
export function onDay(t: Times, day: string): Times {
  return {
    scheduledAt: stampOf(day, timeOf(t.scheduledAt) || '18:00'),
    endsAt: stampOf(day, timeOf(t.endsAt) || '20:00'),
  };
}

/** A new start keeps the length; an end before it moves to an hour after. */
export function withStart(t: Times, time: string): Times {
  const day = dayOf(t.scheduledAt);
  const length = Math.max(15, minutes(timeOf(t.endsAt)) - minutes(timeOf(t.scheduledAt)) || 60);
  const end = Math.min(minutes(time) + length, 23 * 60 + 55);
  return { scheduledAt: stampOf(day, time), endsAt: stampOf(day, clock(end)) };
}

export const withEnd = (t: Times, time: string): Times => ({
  scheduledAt: t.scheduledAt,
  endsAt: stampOf(dayOf(t.scheduledAt), time),
});

/** A meetup's usual weekday (0 Sunday) and times, from its weekly plan (ADR 0119). */
export function usualOf(
  s: EventSchedule | null | undefined,
): { weekday: number; start: string | null; finish: string | null } | null {
  if (!s || s.mode !== 'weekly' || !s.start_date) return null;
  const weekday = new Date(`${s.start_date}T12:00`).getDay();
  return { weekday, start: s.start_time, finish: s.finish_time };
}

/** The next `weekday` on or after `from` (a local day). */
export function nextWeekday(weekday: number, from: string): string {
  const d = new Date(`${from}T12:00`);
  d.setDate(d.getDate() + ((weekday - d.getDay() + 7) % 7));
  return localDayOf(d);
}
