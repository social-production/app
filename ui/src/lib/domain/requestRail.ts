import { t } from '#lib/i18n/index.ts';
import type { RailItem } from './activityRail.ts';
import type { ServiceRequest } from './serviceRequests.ts';

/** web-backend lists the newest six open requests you can review. */
const REQUEST_LIMIT = 6;

/**
 * The right rail's Requests section (web-backend's `kind: "request"` rows, ADR 0076): open
 * requests to services you run, newest first, opening the service with the request.
 */
export function requestRailItems(
  requests: ServiceRequest[],
  me: string,
  names: Record<string, string>,
): RailItem[] {
  return requests
    .filter((r) => r.status === 'open' && r.requester !== me)
    .sort((a, b) => b.createdAt - a.createdAt)
    .slice(0, REQUEST_LIMIT)
    .map((r) => ({
      id: `request:${r.id}`,
      seed: '',
      projectId: r.project,
      kind: 'request',
      title: r.title,
      query: `?request=${encodeURIComponent(r.id)}`,
      meta: r.service,
      createdAt: r.createdAt,
      countLabel: t('Requested by {name}', { name: names[r.requester] ?? t('Unknown requester') }),
      projectMode: 'personal-service',
    }));
}
