import { t } from '#lib/i18n/index.ts';
import { fromWall, wall } from './zone.ts';
import { toLocalInput } from './helpRequests.ts';
import type { RequestSettings } from './projects.ts';

/** Service requests (ADR 0076; web's `ProjectServiceRequestItem`). */

/** web's status ids; "open" shows as Pending. */
export type RequestStatus = 'open' | 'accepted' | 'declined' | 'cancelled';

export interface ServiceRequest {
  id: string;
  /** The direct chat it lives in, and the other person there. */
  chat: string;
  other: string;
  project: string;
  /** The service's title when asked. */
  service: string;
  requester: string;
  title: string;
  body: string;
  status: RequestStatus;
  declineReason: string | null;
  startsAt: number | null;
  endsAt: number | null;
  createdAt: number;
  updatedAt: number;
  /** Who settled it (a collective service's deciding member, ADR 0089). */
  decidedBy: string | null;
}

/** web's `SERVICE_REQUEST_STATUS_LABELS`. */
export const STATUS_LABELS: Record<RequestStatus, string> = {
  open: 'Pending',
  accepted: 'Accepted',
  declined: 'Declined',
  cancelled: 'Cancelled',
};

export const isPending = (status: RequestStatus): boolean => status === 'open';

/** How long a request with no time stays open before it reads Not met (ADR 0113). */
export const REQUEST_OPEN_DAYS = 30;

/** An open request nobody planned before its time (or REQUEST_OPEN_DAYS) passed (ADR 0113).
 * Nothing is written: each device works it out. */
export function notMet(
  r: Pick<ServiceRequest, 'status' | 'endsAt' | 'createdAt'>,
  now = Date.now(),
): boolean {
  if (r.status !== 'open') return false;
  const until = r.endsAt ? r.endsAt / 1000 : r.createdAt / 1000 + REQUEST_OPEN_DAYS * 86_400_000;
  return now > until;
}

/** The title start of a collective request's holder chat (ADR 0089), hidden from Messages. */
export const HOLDER_PREFIX = 'Request · ';

/** The chat a collective request's planner opens with the person asking (ADR 0113): its seed
 * comes from the planner and the request, so anyone who sees the planned activity finds it. */
export const plannersSeed = (planner: string, request: string): string =>
  `${planner}.group.${request.slice(-24)}`;

/** web's `addPreferredTime`: tomorrow 18:00–19:00 in the Display timezone. */
export function preferredTime(now = new Date()): { scheduledAt: string; endsAt: string } {
  const start = wall(now.getTime());
  start.setDate(start.getDate() + 1);
  start.setHours(18, 0, 0, 0);
  const end = new Date(start);
  end.setHours(19);
  return { scheduledAt: toLocalInput(fromWall(start)), endsAt: toLocalInput(fromWall(end)) };
}

/** web's `_ensure_requests_enabled`: why these settings refuse a request with or without a
 * time, or null; `fits` says whether the time lies in an open listed slot (ADR 0077). */
export function refusal(settings: RequestSettings, timed: boolean, fits = true): string | null {
  if (!settings.enabled) return t('Not taking requests.');
  if (settings.mode === 'calendar' && !timed)
    return t('Calendar requests require a start and finish time');
  if (settings.mode === 'direct' && timed && !settings.offSchedule)
    return t('This service only accepts direct unscheduled requests');
  if (timed && !settings.offSchedule && settings.mode !== 'direct' && !fits)
    return t(
      'That time is outside the listed availability slots. Off-schedule requests are turned off.',
    );
  return null;
}

/** web's `requestSettingsSummary`: "Calendar and direct · Off-schedule allowed". */
export function settingsSummary(s: {
  enabled: boolean;
  mode: string;
  offSchedule: boolean;
}): string {
  if (!s.enabled) return 'Disabled';
  const mode =
    s.mode === 'calendar'
      ? 'Calendar only'
      : s.mode === 'direct'
        ? t('Direct only')
        : 'Calendar and direct';
  return `${mode} · ${s.offSchedule ? t('Off-schedule allowed') : t('Slot-bound only')}`;
}
