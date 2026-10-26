/**
 * Activities and roles (ADR 0056), shaped like web's `ProjectActivityItem` as far as the card
 * needs. Times are Holochain microseconds; agents are base64 keys.
 */
export interface ActivityRole {
  index: number;
  label: string;
  required: number;
  maximum: number | null;
  /** Who holds it, in the order they took it. */
  people: string[];
  suggestion: { id: string; person: string; by: string } | null;
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
}

/** web's `minimumParticipants`: every role's required count. */
export const minimumOf = (a: Activity): number => a.roles.reduce((n, r) => n + r.required, 0);

/** web's `committedCount`. */
export const committedOf = (a: Activity): number =>
  a.roles.reduce((n, r) => n + r.people.length, 0);

/** web's "Take role" / "Leave role" / "Role full". */
export function roleButton(role: ActivityRole, mine: boolean): string {
  if (mine) return 'Leave role';
  return role.maximum === null || role.people.length < role.maximum ? 'Take role' : 'Role full';
}

/** web's `roleCapacity`: "0 of 2 filled · 1 more needed". */
export function roleCapacity(role: ActivityRole): string {
  const capacity = role.maximum ?? role.required;
  const short = Math.max(0, role.required - role.people.length);
  const filled = `${role.people.length} of ${capacity} filled`;
  return short > 0 ? `${filled} · ${short} more needed` : filled;
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
    return { label: `${left} ${left === 1 ? 'spot' : 'spots'} left`, tone: 'open' };
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
  if (a.online) return a.place && a.place !== 'Online' ? `Online · ${a.place}` : 'Online';
  return a.place;
}
