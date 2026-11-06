import type { Activity } from './activities.ts';

/**
 * A personal service's listed availability (ADR 0077), ported from web-backend
 * `expand_availability_slots` and `request_fits_open_slot`: weekly hours expand into slots for
 * the next eight weeks in their own time zone, one-off availability is an activity, and an
 * accepted timed request is a booking that takes the slot it sits in. Times are micros.
 */
export interface AvailabilityRule {
  id: string;
  /** 0 = Monday … 6 = Sunday (web's `weekday`). */
  weekday: number;
  /** Minutes after midnight in `zone`. */
  start: number;
  end: number;
  zone: string;
  note: string;
}

export interface Slot {
  id: string;
  /** web's `source`: weekly hours, a one-off activity, or a booking outside any slot. */
  source: 'rule' | 'activity' | 'request';
  title: string;
  startsAt: number;
  endsAt: number;
  held: boolean;
  /** The viewer's own booking covers it. */
  viewerBooked: boolean;
  /** Bookings in it (the creator sees who asked). */
  bookings: Activity[];
}

/** web-backend `WEEKDAY_LABELS`. */
export const WEEKDAYS = [
  'Monday',
  'Tuesday',
  'Wednesday',
  'Thursday',
  'Friday',
  'Saturday',
  'Sunday',
];
export const HORIZON_WEEKS = 8;
const DAY_MS = 86_400_000;

/** web's `_resolve_zone`: an unknown zone reads as UTC. */
export function safeZone(zone: string): string {
  try {
    new Intl.DateTimeFormat('en-US', { timeZone: zone.trim() || 'UTC' });
    return zone.trim() || 'UTC';
  } catch {
    return 'UTC';
  }
}

/** `zone`'s wall clock at instant `ms`, as UTC fields. */
function wall(ms: number, zone: string): number {
  const parts = new Intl.DateTimeFormat('en-US', {
    timeZone: zone,
    hourCycle: 'h23',
    year: 'numeric',
    month: 'numeric',
    day: 'numeric',
    hour: 'numeric',
    minute: 'numeric',
    second: 'numeric',
  }).formatToParts(new Date(ms));
  const n = (type: string): number => Number(parts.find((p) => p.type === type)?.value ?? 0);
  return Date.UTC(n('year'), n('month') - 1, n('day'), n('hour'), n('minute'), n('second'));
}

/** The instant (ms) of `minutes` after midnight on the UTC-field day `day` in `zone`. */
export function zonedInstant(day: number, minutes: number, zone: string): number {
  const target = day + minutes * 60_000;
  let at = target - (wall(target, zone) - target);
  at = target - (wall(at, zone) - at);
  return at;
}

/** "09:00" for 540 minutes. */
export function clockOf(minutes: number): string {
  const h = Math.floor(minutes / 60);
  return `${String(h).padStart(2, '0')}:${String(minutes % 60).padStart(2, '0')}`;
}

/** 540 for "09:00", or null. */
export function minutesOf(clock: string): number | null {
  const m = /^(\d{2}):(\d{2})$/.exec(clock);
  return m ? Number(m[1]) * 60 + Number(m[2]) : null;
}

/** web's rule rows: "Monday 09:00–12:00 (Australia/Melbourne)". */
export function ruleLine(r: AvailabilityRule): string {
  return `${WEEKDAYS[r.weekday] ?? ''} ${clockOf(r.start)}–${clockOf(r.end)} (${r.zone})`;
}

/** Each rule's windows from today (in its zone) for `weeks` weeks that haven't ended. */
export function ruleWindows(
  rules: AvailabilityRule[],
  now = Date.now(),
  weeks = HORIZON_WEEKS,
): { id: string; title: string; startsAt: number; endsAt: number }[] {
  const out = [];
  for (const rule of rules) {
    const zone = safeZone(rule.zone);
    const today = Math.floor(wall(now, zone) / DAY_MS) * DAY_MS;
    for (let i = 0; i <= weeks * 7; i++) {
      const day = today + i * DAY_MS;
      if ((new Date(day).getUTCDay() + 6) % 7 !== rule.weekday) continue;
      const start = zonedInstant(day, rule.start, zone);
      let end = zonedInstant(day, rule.end, zone);
      if (end <= start) end += DAY_MS;
      if (end <= now) continue;
      const date = new Date(day).toISOString().slice(0, 10);
      out.push({
        id: `rule:${rule.id}:${date}`,
        title: rule.note || 'Available',
        startsAt: start * 1000,
        endsAt: end * 1000,
      });
    }
  }
  return out;
}

const overlaps = (a: { startsAt: number; endsAt: number }, b: typeof a): boolean =>
  a.startsAt < b.endsAt && b.startsAt < a.endsAt;
const within = (inner: { startsAt: number; endsAt: number }, outer: typeof inner): boolean =>
  inner.startsAt >= outer.startsAt && inner.endsAt <= outer.endsAt;

/** web-backend `expand_availability_slots`, soonest first. */
export function serviceSlots(
  rules: AvailabilityRule[],
  activities: Activity[],
  me: string,
  now = Date.now(),
): Slot[] {
  const bookings = activities.filter((a) => a.booking);
  const open = [
    ...ruleWindows(rules, now).map((w) => ({ ...w, source: 'rule' as const })),
    ...activities
      .filter((a) => !a.booking && a.endsAt / 1000 > now)
      .map((a) => ({
        id: a.id,
        title: a.title,
        startsAt: a.startsAt,
        endsAt: a.endsAt,
        source: 'activity' as const,
      })),
  ];
  // web holds a slot when a booking that fits an open slot is accepted.
  const holds = bookings.filter((b) => open.some((s) => within(b, s)));
  const mine = (b: Activity): boolean => b.booking?.requester === me;
  const slots: Slot[] = open.map((s) => ({
    ...s,
    held: holds.some((h) => overlaps(h, s)),
    viewerBooked: bookings.some((b) => mine(b) && overlaps(b, s)),
    bookings: bookings.filter((b) => overlaps(b, s)),
  }));
  for (const b of bookings) {
    if (b.endsAt / 1000 <= now || open.some((s) => within(b, s))) continue;
    slots.push({
      id: b.id,
      source: 'request',
      title: b.title,
      startsAt: b.startsAt,
      endsAt: b.endsAt,
      held: true,
      viewerBooked: mine(b),
      bookings: [],
    });
  }
  return slots.sort((x, y) => x.startsAt - y.startsAt);
}

/** web-backend `request_fits_open_slot`: an open listed slot contains the time. */
export const fitsOpenSlot = (slots: Slot[], startsAt: number, endsAt: number): boolean =>
  slots.some((s) => s.source !== 'request' && !s.held && within({ startsAt, endsAt }, s));

/** The first open slot on the local day `iso` (web's `firstAvailabilityForDay`). */
export function firstSlotOn(slots: Slot[], iso: string): Slot | undefined {
  const dayStart = new Date(`${iso}T00:00:00`).getTime() * 1000;
  const dayEnd = new Date(`${iso}T23:59:59`).getTime() * 1000;
  return slots.find((s) => !s.held && s.startsAt <= dayEnd && s.endsAt >= dayStart);
}
