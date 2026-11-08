import type { RegionItem, RegionQuery } from '#lib/domain/regionFeed.ts';
import { placeIndex } from '#lib/holo/index.ts';

/** web's `getRegionFeedPage` against the gateway's `/index/region` (ADR 0084). */
export function regionFeed(q: RegionQuery): Promise<{ items: RegionItem[]; total: number }> {
  return placeIndex('region', {
    lat: String(q.lat),
    lon: String(q.lon),
    radius_km: String(Math.round(q.radiusKm)),
    sort: q.sort,
    window: q.window,
    filter: q.filter,
    include_online: String(q.includeOnline),
    offset: String(q.offset),
    limit: '20',
    tzOffset: String(-new Date().getTimezoneOffset()),
  });
}
