import { markerItem, type GatewayMarker, type MapMarkerItem } from '#lib/domain/mapMarkers.ts';
import { placeIndex } from '#lib/holo/index.ts';

/** web's `getMapMarkers` against the gateway's `/index/map-markers` (ADR 0082, 0083, 0085). */
export interface MarkerQuery {
  lat: number;
  lon: number;
  radiusKm?: number;
  window?: string;
  filter?: string;
  dateFrom?: string;
  dateTo?: string;
  upcomingOnly?: boolean;
}

export async function getMapMarkers(q: MarkerQuery): Promise<MapMarkerItem[]> {
  const params: Record<string, string> = {
    lat: String(q.lat),
    lon: String(q.lon),
    radius_km: String(q.radiusKm ?? 25),
    // The viewer's day for the window (minutes east of UTC), in place of web's zone name.
    tzOffset: String(-new Date().getTimezoneOffset()),
  };
  if (q.window) params.window = q.window;
  if (q.filter) params.filter = q.filter;
  if (q.dateFrom) params.date_from = q.dateFrom;
  if (q.dateTo) params.date_to = q.dateTo;
  if (q.upcomingOnly !== undefined) params.upcoming_only = q.upcomingOnly ? 'true' : 'false';
  const res = await placeIndex<{ items?: GatewayMarker[] }>('map-markers', params);
  return (res.items ?? []).map(markerItem);
}
