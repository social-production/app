/** web's pin stacking (`maps/maplibre.ts`): a parent and its activities at one spot share a
 * pin; other items there stack; pins closer than 40 px shrink to dots. */
import type { MapMarker } from './types.ts';

const OVERLAP_PIXEL_THRESHOLD = 40;

function scheduleSortKey(item: MapMarker): number {
  if (!item.scheduledAt) {
    return Number.MAX_SAFE_INTEGER;
  }
  const parsed = Date.parse(item.scheduledAt);
  return Number.isFinite(parsed) ? parsed : Number.MAX_SAFE_INTEGER;
}

function isParentEntity(item: MapMarker): boolean {
  return (
    item.entityType === 'event' ||
    item.entityType === 'project' ||
    item.entityType === 'help_request'
  );
}

/**
 * Same-location parent + activity markers collapse into one parent pin with nested sub-pins.
 * Unrelated same-location items still stack vertically as expanded pins.
 */
interface Stacked {
  markers: MapMarker[];
  forceExpandedIds: Set<string>;
  childToParentId: Map<string, string>;
}

const STACK_LAT_STEP = 0.00038;
const bySchedule = (a: MapMarker, b: MapMarker): number =>
  scheduleSortKey(a) - scheduleSortKey(b) || a.id.localeCompare(b.id);

/** One spot's markers: each parent with its activities, then the rest, one under another. */
function stackGroup(group: MapMarker[], out: Stacked): void {
  const activities = group.filter((item) => item.entityType === 'activity');
  const used = new Set<string>();
  const entries: MapMarker[] = [];
  for (const parent of group.filter(isParentEntity).sort(bySchedule)) {
    const children = activities.filter((item) => item.parentId === parent.id).sort(bySchedule);
    used.add(parent.id);
    for (const child of children) {
      used.add(child.id);
      out.childToParentId.set(child.id, parent.id);
    }
    entries.push(children.length > 0 ? { ...parent, children } : parent);
    out.forceExpandedIds.add(parent.id);
  }
  for (const item of group.filter((m) => !used.has(m.id)).sort(bySchedule)) {
    entries.push(item);
    out.forceExpandedIds.add(item.id);
  }
  entries.forEach((item, index) => {
    out.markers.push({ ...item, latitude: item.latitude - STACK_LAT_STEP * index });
  });
}

export function stackOverlappingCoordinates(nextMarkers: MapMarker[]): Stacked {
  const groups = new Map<string, MapMarker[]>();
  for (const item of nextMarkers) {
    const key = `${item.latitude.toFixed(5)}:${item.longitude.toFixed(5)}`;
    const bucket = groups.get(key);
    if (bucket) bucket.push(item);
    else groups.set(key, [item]);
  }
  const out: Stacked = { markers: [], forceExpandedIds: new Set(), childToParentId: new Map() };
  for (const group of groups.values()) {
    if (group.length === 1) out.markers.push(...group);
    else stackGroup(group, out);
  }
  return out;
}

export function detectCrowdedMarkerIds(
  project: (lngLat: [number, number]) => { x: number; y: number },
  nextMarkers: MapMarker[],
  forceExpandedIds: Set<string> = new Set(),
): Set<string> {
  const crowded = new Set<string>();
  const projected = nextMarkers
    .filter((item) => !forceExpandedIds.has(item.id))
    .map((item) => ({ id: item.id, ...project([item.longitude, item.latitude]) }));
  const thresholdSq = OVERLAP_PIXEL_THRESHOLD * OVERLAP_PIXEL_THRESHOLD;
  projected.forEach((a, i) => {
    for (const b of projected.slice(i + 1)) {
      const dx = a.x - b.x;
      const dy = a.y - b.y;
      if (dx * dx + dy * dy <= thresholdSq) {
        crowded.add(a.id);
        crowded.add(b.id);
      }
    }
  });
  return crowded;
}
