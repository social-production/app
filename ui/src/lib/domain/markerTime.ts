/** web's map times (`utils/time.ts` `formatMarkerScheduleRange`, `formatLocalDateTimeRange`):
 * a pin's or card's start–end, with a +2 h end when it has none. Browser time zone, as the
 * app's other times (PARITY: display timezone). */
import { zoneOf } from './activityTime.ts';

const DEFAULT_MARKER_DURATION_MS = 2 * 60 * 60 * 1000;
const dateOptions: Intl.DateTimeFormatOptions = {
  weekday: 'short',
  month: 'short',
  day: 'numeric',
  year: 'numeric',
};
const timeOptions: Intl.DateTimeFormatOptions = { hour: 'numeric', minute: '2-digit' };

const valid = (iso: string | null | undefined): Date | null => {
  const d = iso?.trim() ? new Date(iso) : null;
  return d && !Number.isNaN(d.getTime()) ? d : null;
};

function full(d: Date): string {
  const zone = zoneOf(d);
  const text = d.toLocaleString(undefined, { ...dateOptions, ...timeOptions });
  return zone ? `${text} ${zone}` : text;
}

/** web's `effectiveEndsAt`. */
export function effectiveEndsAt(
  scheduledAt: string | null | undefined,
  endsAt: string | null | undefined,
): string | null {
  const start = valid(scheduledAt);
  if (!start) return null;
  if (endsAt && valid(endsAt)) return endsAt.trim();
  return new Date(start.getTime() + DEFAULT_MARKER_DURATION_MS).toISOString();
}

export function formatMarkerScheduleRange(
  scheduledAt: string | null | undefined,
  endsAt: string | null | undefined,
): string {
  const start = valid(scheduledAt);
  if (!start) return '';
  const end = valid(effectiveEndsAt(scheduledAt, endsAt));
  if (!end) return full(start);
  if (start.toDateString() !== end.toDateString()) return `${full(start)} – ${full(end)}`;
  const zone = zoneOf(end);
  const day = start.toLocaleDateString(undefined, dateOptions);
  const range = `${day}, ${start.toLocaleTimeString(undefined, timeOptions)} – ${end.toLocaleTimeString(undefined, timeOptions)}`;
  return zone ? `${range} ${zone}` : range;
}
