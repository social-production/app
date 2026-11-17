/**
 * Dates and times on activity cards, as web's CollapsibleActivityCard and ActivityAgendaList
 * write them, in the Display timezone (`zone.ts`). Activity times are Holochain microseconds.
 */
import { inZone, wall, zoneName } from './zone.ts';
const at = (micros: number): Date => new Date(micros / 1000);

/** web's `formatLocalTimezoneAbbreviation`: "AEDT", or "" when the browser gives only GMT±n. */
export function zoneOf(date: Date): string {
  return zoneName(date.getTime());
}

const isoDay = (date: Date): string =>
  `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;

export const dayKey = (micros: number): string => isoDay(wall(micros / 1000));

export function clockTime(micros: number): string {
  return at(micros).toLocaleTimeString(undefined, inZone({ hour: 'numeric', minute: '2-digit' }));
}

/** "Sat 18" when an activity runs past midnight, else "". */
export function multiDayEnd(start: number, end: number): string {
  if (dayKey(start) === dayKey(end)) return '';
  return at(end)
    .toLocaleDateString('en-GB', inZone({ weekday: 'short', day: 'numeric' }))
    .replace(',', '');
}

/** web's `dateLine`: "Sat, 18 Oct · 10:00–12:00 AEDT", or both days when it spans midnight. */
export function dateLine(start: number, end: number): string {
  const s = at(start);
  const e = at(end);
  const day = inZone({ weekday: 'short', month: 'short', day: 'numeric' });
  const time = inZone({ hour: 'numeric', minute: '2-digit' });
  const zone = zoneOf(s);
  const suffix = zone ? ` ${zone}` : '';
  const startDate = s.toLocaleDateString(undefined, day);
  const startTime = s.toLocaleTimeString(undefined, time);
  const endTime = e.toLocaleTimeString(undefined, time);
  if (dayKey(start) === dayKey(end)) return `${startDate} · ${startTime}–${endTime}${suffix}`;
  const endDate = e.toLocaleDateString(undefined, day);
  return `${startDate}, ${startTime} – ${endDate}, ${endTime}${suffix}`;
}

/** web's agenda day heading: "Today · Sat, 18 Oct", "Tomorrow · …" or the day. */
export function dayHeading(micros: number, now = Date.now()): string {
  const date = at(micros);
  const base = date.toLocaleDateString(
    undefined,
    inZone({ weekday: 'short', day: 'numeric', month: 'short' }),
  );
  const day = dayKey(micros);
  if (day === isoDay(wall(now))) return `Today · ${base}`;
  if (day === isoDay(wall(now + 24 * 60 * 60 * 1000))) return `Tomorrow · ${base}`;
  return base;
}
