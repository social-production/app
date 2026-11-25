/**
 * A tapped pin's sessions (Map cards mockup v2, owner 2026-11-25): a meetup's, event's or
 * project's upcoming activities on the map. A tapped session comes first, then others at
 * its spot, then the rest by time; a tapped parent lists them by time.
 */
import type { MapMarkerItem } from '#lib/domain/mapMarkers.ts';

const startOf = (i: MapMarkerItem): number => Date.parse(i.scheduledAt ?? '') || Infinity;
const notOver = (i: MapMarkerItem, now: number): boolean =>
  !(Date.parse(i.endsAt ?? i.scheduledAt ?? '') < now);
const sameSpot = (a: MapMarkerItem, b: MapMarkerItem): boolean =>
  a.latitude === b.latitude && a.longitude === b.longitude;

export function sessionsFor(
  items: MapMarkerItem[],
  picked: MapMarkerItem,
  now = Date.now(),
): MapMarkerItem[] {
  const parent = picked.entityType === 'activity' ? picked.parentId : picked.id;
  if (!parent) return [];
  const rank = (i: MapMarkerItem): number =>
    picked.entityType !== 'activity' ? 0 : i.id === picked.id ? 0 : sameSpot(i, picked) ? 1 : 2;
  return items
    .filter((i) => i.entityType === 'activity' && i.parentId === parent && notOver(i, now))
    .sort((a, b) => rank(a) - rank(b) || startOf(a) - startOf(b));
}

/** The marker a pin's card is about: the parent when it's on the map, else the session. */
export function headOf(items: MapMarkerItem[], picked: MapMarkerItem): MapMarkerItem {
  if (picked.entityType !== 'activity' || !picked.parentId) return picked;
  return items.find((i) => i.id === picked.parentId) ?? picked;
}

/** A tapped pin's card: what it's about (a session's parent) and its sessions. */
export function tappedCard(
  items: MapMarkerItem[],
  id: string | null,
): { head: MapMarkerItem; sessions: MapMarkerItem[] } | null {
  const picked = id ? items.find((i) => i.id === id) : undefined;
  return picked ? { head: headOf(items, picked), sessions: sessionsFor(items, picked) } : null;
}
