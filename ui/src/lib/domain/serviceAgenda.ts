import type { Activity } from './activities.ts';
import type { Slot } from './availability.ts';

/**
 * web's IndividualServicePhaseOne agenda and counts (ADR 0077): everyone but the creator sees
 * the service's slots as agenda rows marked Yours / Taken / Closed / Open; the creator sees
 * their activities marked Booked / Closed / Open.
 */
export type Chip = { label: string; tone: 'locked' | 'need' | 'full' | 'open' };

/** web's `calendarActivities` title for a slot. */
export function slotTitle(slot: Slot, creator: boolean): string {
  if (!slot.held) return slot.title;
  if (creator) return `${slot.title} (taken)`;
  return slot.viewerBooked ? 'Your booking' : 'Taken';
}

/** A slot as an agenda row / calendar bar (web maps slots to `ProjectActivityItem`s). */
export function slotActivity(
  slot: Slot,
  creator: boolean,
  author: string,
  place: string,
  names: Record<string, string>,
): Activity {
  return {
    id: slot.id,
    author,
    createdAt: 0,
    title: slotTitle(slot, creator),
    startsAt: slot.startsAt,
    endsAt: slot.endsAt,
    online: !place,
    place: place || 'Online',
    meetingUrl: null,
    note: slot.bookings
      .map((b) => (b.booking ? `@${names[b.booking.requester] ?? 'someone'}` : ''))
      .filter(Boolean)
      .join(', '),
    planStage: null,
    roles: [],
    myRole: null,
    rolesOpen: !slot.held,
    ratings: [],
    canRate: false,
    completion: {
      eligible: 0,
      completed: 0,
      uncompleted: 0,
      mine: null,
      canSet: false,
      requester: null,
      requesterCanSet: false,
    },
    booking: null,
  };
}

/** The creator's row chip: a booking is Booked, anything else Open (Closed while off). */
export function creatorChip(a: Activity, requestsOff: boolean): Chip {
  if (a.booking) return { label: 'Booked', tone: 'full' };
  return requestsOff ? { label: 'Closed', tone: 'locked' } : { label: 'Open', tone: 'open' };
}

/** Everyone else's row chip for a slot. */
export function slotChip(slot: Slot, requestsOff: boolean): Chip {
  if (slot.viewerBooked) return { label: 'Yours', tone: 'open' };
  if (slot.held) return { label: 'Taken', tone: 'full' };
  return requestsOff ? { label: 'Closed', tone: 'locked' } : { label: 'Open', tone: 'open' };
}

const plural = (n: number, one: string, many: string): string => `${n} ${n === 1 ? one : many}`;

/** web's Activity section line ("2 open slots · 1 taken (1 yours)"). */
export function countsLine(o: {
  usesCalendar: boolean;
  requestsOff: boolean;
  creator: boolean;
  slots: Slot[];
  accepted: number;
  live: number;
  now?: number;
}): string {
  const now = (o.now ?? Date.now()) * 1000;
  const future = o.slots.filter((s) => s.endsAt > now);
  const open = future.filter((s) => !s.held && s.source !== 'request').length;
  const taken = future.filter((s) => s.held).length;
  const yours = future.filter((s) => s.viewerBooked).length;
  if (!o.usesCalendar) return plural(o.live, 'future activity', 'future activities');
  const side = o.creator
    ? plural(o.accepted, 'accepted request', 'accepted requests')
    : `${taken} taken${!o.requestsOff && yours > 0 ? ` (${yours} yours)` : ''}`;
  return o.requestsOff
    ? `Not taking requests · ${side}`
    : `${plural(open, 'open slot', 'open slots')} · ${side}`;
}

/** web's `futureOpenSlotCount`. */
export const openSlots = (slots: Slot[], now = Date.now()): number =>
  slots.filter((s) => !s.held && s.source !== 'request' && s.endsAt > now * 1000).length;
