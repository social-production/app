/** web's Region feed (`/feeds/region`) from the gateway's `/index/region` (ADR 0084): feed
 * cards with their distance, and web's project-activity items. */
import type { ProjectCard } from './projects.ts';

export interface RegionCard extends ProjectCard {
  /** km from the centre; null for an online item (appended after the page). */
  distanceKm: number | null;
  isOnline: boolean;
}

/** web's `PublicProjectActivityItem` (times in Holochain microseconds, as other cards). */
export interface RegionActivity {
  kind: 'project-activity';
  id: string;
  parentId: string;
  href: string;
  title: string;
  parentTitle: string;
  projectMode: string;
  locationLabel: string;
  scheduledAt: number;
  endsAt: number;
  createdAt: number;
  lastActivityAt: number;
  distanceKm: number;
}

export type RegionItem = RegionCard | RegionActivity;

export const isActivity = (item: RegionItem): item is RegionActivity =>
  (item as RegionActivity).kind === 'project-activity';

export interface RegionQuery {
  lat: number;
  lon: number;
  radiusKm: number;
  sort: string;
  window: string;
  filter: string;
  includeOnline: boolean;
  offset: number;
}

/** web's `openRegionalFeed` link: `/?scope=region&lat=&lon=&place=&radius=&window=&filter=`. */
export interface RegionLink {
  lat: number;
  lon: number;
  place: string;
  radius: string | null;
  window: string | null;
}

export function regionLink(params: { get(name: string): string | null }): RegionLink | null {
  if (params.get('scope') !== 'region') return null;
  const lat = Number(params.get('lat'));
  const lon = Number(params.get('lon'));
  if (!params.get('lat') || !params.get('lon') || !Number.isFinite(lat) || !Number.isFinite(lon)) {
    return null;
  }
  return {
    lat,
    lon,
    place: params.get('place') ?? '',
    radius: params.get('radius'),
    window: params.get('window'),
  };
}

/** web's `openRegionalFeed`: the map's place and filters as a feed link. */
export function regionLinkQuery(at: {
  lat: number;
  lon: number;
  place: string;
  radius: string;
  window: string;
  filter: string;
}): string {
  const params = new URLSearchParams({
    scope: 'region',
    lat: String(at.lat),
    lon: String(at.lon),
    ...(at.place.trim() ? { place: at.place.trim() } : {}),
    radius: at.radius,
    window: at.window,
    filter: at.filter,
  });
  return `?${params.toString()}`;
}
