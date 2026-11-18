import { t } from '#lib/i18n/index.ts';
import {
  notMet,
  REQUEST_OPEN_DAYS,
  STATUS_LABELS,
  type ServiceRequest,
} from './serviceRequests.ts';
import type { NotificationItem } from './notifications.ts';

type Unread = Omit<NotificationItem, 'isUnread'>;

/**
 * web-backend's `svc-request` cards (ADR 0076, `_notify_service_request`): a new request
 * tells the service's creator; each decision tells the other person ("Request accepted: …"),
 * and a request nobody planned in time tells the person asking ("Request not met: …", ADR 0113).
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
      body: `${t('Request {status}', { status: t(STATUS_LABELS[status]).toLowerCase() })}: ${r.title}`,
      lead: null,
      target: { request: { project: r.project, id: r.id } },
      createdAt: at,
    });
    const out: Unread[] = [];
    if (!mine) out.push(card('open', r.requester, r.createdAt));
    // Nobody planned it in time (ADR 0113): the person asking hears once, at the cut-off.
    if (mine && notMet(r)) {
      const at = r.endsAt ?? r.createdAt + REQUEST_OPEN_DAYS * 86_400_000_000;
      out.push({
        ...card('open', r.other, at),
        id: `svc-request:${r.id}:not-met`,
        body: `${t('Request not met')}: ${r.title}`,
      });
    }
    // Who decided: the person asking cancels; the creator accepts or declines.
    const byRequester = r.status === 'cancelled';
    if (r.status !== 'open' && byRequester !== mine) {
      out.push(card(r.status, r.decidedBy ?? (byRequester ? r.requester : r.other), r.updatedAt));
    }
    return out;
  });
}
