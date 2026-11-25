/**
 * The rail's day groups and time chips (Rail and Group Chat mockup §1, owner 2026-11-25):
 * cards under "Today", "Tomorrow", a weekday or a date, each with its start time in a chip.
 * Times are Holochain microseconds, shown in the Display timezone.
 */
import { locale, t } from '#lib/i18n/index.ts';
import { displayZone } from './zone.ts';

/** The calendar day of `ms` in `zone`, as "YYYY-MM-DD". */
function dayKey(ms: number, zone: string): string {
  return new Date(ms).toLocaleDateString('en-CA', { timeZone: zone });
}

const DAY_MS = 24 * 60 * 60 * 1000;

/** "Today", "Tomorrow", "Saturday" within the week, then "Mon 8 Dec". */
export function railDay(micros: number, nowMs = Date.now(), zone = displayZone()): string {
  const ms = micros / 1000;
  const key = dayKey(ms, zone);
  if (key === dayKey(nowMs, zone)) return t('Today');
  if (key === dayKey(nowMs + DAY_MS, zone)) return t('Tomorrow');
  const days = (Date.parse(key) - Date.parse(dayKey(nowMs, zone))) / DAY_MS;
  const opts: Intl.DateTimeFormatOptions =
    days > 0 && days < 7
      ? { weekday: 'long', timeZone: zone }
      : { weekday: 'short', day: 'numeric', month: 'short', timeZone: zone };
  return new Date(ms).toLocaleDateString(locale(), opts);
}

/** Consecutive runs of items on the same day, in their order. */
export function byRailDay<T>(items: T[], at: (item: T) => number): { day: string; items: T[] }[] {
  const groups: { day: string; items: T[] }[] = [];
  for (const item of items) {
    const day = railDay(at(item));
    const last = groups.at(-1);
    if (last?.day === day) last.items.push(item);
    else groups.push({ day, items: [item] });
  }
  return groups;
}

/** The chip's two lines: "5:00" over "AM" (or "17:00" alone where clocks run to 24). */
export function railClock(micros: number, zone = displayZone()): { time: string; half: string } {
  const parts = new Intl.DateTimeFormat(locale(), {
    hour: 'numeric',
    minute: '2-digit',
    timeZone: zone,
  }).formatToParts(new Date(micros / 1000));
  const half = parts.find((p) => p.type === 'dayPeriod')?.value ?? '';
  const time = parts
    .filter((p) => p.type !== 'dayPeriod')
    .map((p) => p.value)
    .join('')
    .trim();
  return { time, half };
}
