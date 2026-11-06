import { STATUS_LABELS, type ServiceRequest } from './serviceRequests.ts';
import type { NotificationItem } from './notifications.ts';

type Unread = Omit<NotificationItem, 'isUnread'>;

/**
 * web-backend's `svc-request` cards (ADR 0076, `_notify_service_request`): a new request
 * tells the service's creator; each decision tells the other person ("Request accepted: …").
 * Read from the direct chats the requests live in.
 */
export function requestNotifications(requests: ServiceRequest[], me: string): Unread[] {
  return requests.flatMap((r) => {
    const mine = r.requester === me;
    const card = (status: ServiceRequest['status'], actor: string, at: number): Unread => ({
      id: `svc-request:${r.id}:${status}`,
      kind: 'svc-request',
      subject: 'project',
      seed: '',
      projectMode: 'personal-service',
      actor,
      title: r.service,
      body: `Request ${STATUS_LABELS[status].toLowerCase()}: ${r.title}`,
      lead: null,
      target: { request: { project: r.project, id: r.id } },
      createdAt: at,
    });
    const out: Unread[] = [];
    if (!mine) out.push(card('open', r.requester, r.createdAt));
    // Who decided: the person asking cancels; the creator accepts or declines.
    const byRequester = r.status === 'cancelled';
    if (r.status !== 'open' && byRequester !== mine) {
      out.push(card(r.status, byRequester ? r.requester : r.other, r.updatedAt));
    }
    return out;
  });
}
