/**
 * The map's results as the feed draws them (owner, 2026-11-25: "it looks right in the Global
 * feed but under map needs some work"): the Region feed's cards for the same area, by id, and
 * the markers with a meetup's, event's or project's sessions folded into their parent's card
 * (which shows the next session) whenever the parent is on the map too.
 */
import { isActivity, type RegionCard } from '#lib/domain/regionFeed.ts';
import type { MapMarkerItem } from '#lib/domain/mapMarkers.ts';
import { regionFeed } from '#lib/services/region.ts';

/** The map's filter in the Region feed's words ("all" is no filter). */
const regionFilter = (filter: string): string => (filter === 'all' ? '' : filter);
/** A custom date range has no Region equivalent; its cards come from "all". */
const regionWindow = (window: string): string => (window === 'custom' ? 'all' : window);

export async function mapCards(q: {
  lat: number;
  lon: number;
  radiusKm: number;
  window: string;
  filter: string;
}): Promise<Record<string, RegionCard>> {
  const { items } = await regionFeed({
    lat: q.lat,
    lon: q.lon,
    radiusKm: q.radiusKm,
    sort: '',
    window: regionWindow(q.window),
    filter: regionFilter(q.filter),
    includeOnline: false,
    order: 'nearest',
    offset: 0,
    limit: 100,
  });
  const cards: Record<string, RegionCard> = {};
  for (const item of items) if (!isActivity(item)) cards[item.id] = item;
  return cards;
}

/** The markers without the sessions whose parent is on the map too (one pin, one card). */
export function foldSessions(items: MapMarkerItem[]): MapMarkerItem[] {
  const shown = new Set(items.filter((i) => i.entityType !== 'activity').map((i) => i.id));
  return items.filter((i) => i.entityType !== 'activity' || !i.parentId || !shown.has(i.parentId));
}

/** The card a marker is about: its own, or its parent's for a session. */
export function cardOf(
  cards: Record<string, RegionCard>,
  item: Pick<MapMarkerItem, 'id' | 'entityType' | 'parentId'>,
): RegionCard | null {
  if (item.entityType === 'activity') return item.parentId ? (cards[item.parentId] ?? null) : null;
  return cards[item.id] ?? null;
}
