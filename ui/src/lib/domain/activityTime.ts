/**
 * Dates and times on activity cards, as web's CollapsibleActivityCard and ActivityAgendaList
 * write them. web follows its Display timezone setting; the app has none yet, so times are the
 * browser's (PARITY: display timezone). Activity times are Holochain microseconds.
 */
const at = (micros: number): Date => new Date(micros / 1000);

/** web's `formatLocalTimezoneAbbreviation`: "AEDT", or "" when the browser gives only GMT±n. */
export function zoneOf(date: Date): string {
  const zone = Intl.DateTimeFormat().resolvedOptions().timeZone;
  const locale = zone.startsWith('Australia/') ? 'en-AU' : undefined;
  const part = new Intl.DateTimeFormat(locale, { timeZoneName: 'short' })
    .formatToParts(date)
    .find((p) => p.type === 'timeZoneName')?.value;
  return part && !/^GMT[+-]/i.test(part) ? part : '';
}

const isoDay = (date: Date): string =>
  `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;

export const dayKey = (micros: number): string => isoDay(at(micros));

export function clockTime(micros: number): string {
  return at(micros).toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' });
}

/** "Sat 18" when an activity runs past midnight, else "". */
export function multiDayEnd(start: number, end: number): string {
  if (dayKey(start) === dayKey(end)) return '';
  return at(end).toLocaleDateString('en-GB', { weekday: 'short', day: 'numeric' }).replace(',', '');
}

/** web's `dateLine`: "Sat, 18 Oct · 10:00–12:00 AEDT", or both days when it spans midnight. */
export function dateLine(start: number, end: number): string {
  const s = at(start);
  const e = at(end);
  const day: Intl.DateTimeFormatOptions = { weekday: 'short', month: 'short', day: 'numeric' };
  const time: Intl.DateTimeFormatOptions = { hour: 'numeric', minute: '2-digit' };
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
  const base = date.toLocaleDateString(undefined, {
    weekday: 'short',
    day: 'numeric',
    month: 'short',
  });
  const day = isoDay(date);
  if (day === isoDay(new Date(now))) return `Today · ${base}`;
  if (day === isoDay(new Date(now + 24 * 60 * 60 * 1000))) return `Tomorrow · ${base}`;
  return base;
}
