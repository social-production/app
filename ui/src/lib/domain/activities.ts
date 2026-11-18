import { t } from '#lib/i18n/index.ts';
import { plural } from '#lib/i18n/index.ts';
/**
 * Activities and roles (ADR 0056), shaped like web's `ProjectActivityItem` as far as the card
 * needs. Times are Holochain microseconds; agents are base64 keys.
 */
export interface ActivityRole {
  index: number;
  label: string;
  required: number;
  maximum: number | null;
  /** A help request's role description (ADR 0067); empty elsewhere. */
  description: string;
  /** Who holds it, in the order they took it, and when each took it (micros). */
  people: string[];
  takenAt: number[];
  suggestion: RoleSuggestion | null;
  /** Suggestions their person turned down (the decline's id and time). */
  declined: RoleSuggestion[];
}

export interface RoleSuggestion {
  id: string;
  person: string;
  by: string;
  at: number;
}

export interface ActivityRating {
  author: string;
  stars: number;
  comment: string;
  at: number;
}

/** Participants' check-ins once it has ended (web's participant completion side). */
export interface ActivityCompletion {
  eligible: number;
  completed: number;
  uncompleted: number;
  mine: boolean | null;
  canSet: boolean;
  /** A booking's requester side (ADR 0077): their check-in, and whether you are them. */
  requester: boolean | null;
  requesterCanSet: boolean;
}

/** The service request an activity books (ADR 0077) and who asked. */
export interface Booking {
  request: string;
  requester: string;
}

export interface Activity {
  id: string;
  author: string;
  createdAt: number;
  title: string;
  startsAt: number;
  endsAt: number;
  online: boolean;
  place: string;
  meetingUrl: string | null;
  note: string;
  planStage: string | null;
  roles: ActivityRole[];
  myRole: number | null;
  rolesOpen: boolean;
  ratings: ActivityRating[];
  canRate: boolean;
  completion: ActivityCompletion;
  booking: Booking | null;
}

/** web's `minimumParticipants`: every role's required count. */
export const minimumOf = (a: Activity): number => a.roles.reduce((n, r) => n + r.required, 0);

/** web's `committedCount`. */
export const committedOf = (a: Activity): number =>
  a.roles.reduce((n, r) => n + r.people.length, 0);

/** A role's pill (Feed Headers and Activity Phase §3, as a help request's): "Join", "Leave",
 * or "Full" once every place is taken. */
export function roleButton(role: ActivityRole, mine: boolean): 'Join' | 'Leave' | 'Full' {
  if (mine) return 'Leave';
  return role.maximum === null || role.people.length < role.maximum ? 'Join' : 'Full';
}

/** A role's line under its name: "Mika, Jo · 2 of 2", or "0 of 2" while nobody is in. */
export function roleLine(role: ActivityRole, nameOf: (agent: string) => string): string {
  const count = `${role.people.length} of ${role.maximum ?? role.required}`;
  return role.people.length ? `${role.people.map(nameOf).join(', ')} · ${count}` : count;
}

/** web's agenda chip (CollapsibleActivityCard `fillChipLabel` / `fillChipTone`). */
export function fillChip(a: Activity): {
  label: string;
  tone: 'locked' | 'need' | 'full' | 'open';
} {
  if (!a.rolesOpen) return { label: 'Ended', tone: 'locked' };
  const committed = committedOf(a);
  const needed = Math.max(0, minimumOf(a) - committed);
  if (needed > 0) return { label: `${needed} more needed`, tone: 'need' };
  const capped = a.roles.every((r) => r.maximum !== null);
  const left = capped
    ? Math.max(0, a.roles.reduce((n, r) => n + (r.maximum ?? 0), 0) - committed)
    : null;
  if (left === 0) return { label: 'Full', tone: 'full' };
  if (left !== null)
    return { label: `${plural(left, '{n} spot', '{n} spots')} left`, tone: 'open' };
  return { label: `${committed} going`, tone: 'open' };
}

/** web's status stripe: red nobody yet, yellow below the minimum, green minimum met. */
export function signupTone(a: Activity): 'empty' | 'partial' | 'met' {
  const committed = committedOf(a);
  if (committed <= 0) return 'empty';
  return committed >= minimumOf(a) ? 'met' : 'partial';
}

/** web's place line: "Online", "Online · Zoom" or the place. */
export function placeLabel(a: Activity): string {
  if (a.online)
    return a.place && a.place !== 'Online' ? `${t('Online')} · ${a.place}` : t('Online');
  return a.place;
}
