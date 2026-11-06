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
}

/** web's `SERVICE_REQUEST_STATUS_LABELS`. */
export const STATUS_LABELS: Record<RequestStatus, string> = {
  open: 'Pending',
  accepted: 'Accepted',
  declined: 'Declined',
  cancelled: 'Cancelled',
};

export const isPending = (status: RequestStatus): boolean => status === 'open';

/** web's `addPreferredTime`: tomorrow 18:00–19:00 in the browser's zone. */
export function preferredTime(now = new Date()): { scheduledAt: string; endsAt: string } {
  const start = new Date(now);
  start.setDate(start.getDate() + 1);
  start.setHours(18, 0, 0, 0);
  const end = new Date(start);
  end.setHours(19);
  return { scheduledAt: toLocalInput(start), endsAt: toLocalInput(end) };
}

/** web's `_ensure_requests_enabled` (without listed slots, ADR 0076 part b): why these
 * settings refuse a request with or without a time, or null. */
export function refusal(settings: RequestSettings, timed: boolean): string | null {
  if (!settings.enabled) return 'Not taking requests.';
  if (settings.mode === 'calendar' && !timed)
    return 'Calendar requests require a start and finish time';
  if (settings.mode === 'direct' && timed && !settings.offSchedule)
    return 'This service only accepts direct unscheduled requests';
  return null;
}
