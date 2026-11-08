/** web's pin layer (`maps/maplibre.ts` `renderMarkers`, `scheduleCompactRefresh`): stacked,
 * crowded pins shrink to dots, the highlighted one stays expanded. */
import { Marker, type Map as MapLibreMap } from 'maplibre-gl';
import { buildMarkerElement, type PinContext } from './markerElement.ts';
import { detectCrowdedMarkerIds, stackOverlappingCoordinates } from './stacking.ts';
import type { MapMarker } from './types.ts';

export class Pins {
  pending: MapMarker[] = [];
  private placed: Marker[] = [];
  private rendering = false;
  private lastCrowdedKey = '';
  private compactDebounce: ReturnType<typeof setTimeout> | null = null;
  private highlightedId: string | null = null;
  private byId = new Map<string, { marker: Marker; item: MapMarker }>();
  private forceExpanded = new Set<string>();
  private childToParent = new Map<string, string>();

  constructor(
    private readonly map: () => MapLibreMap | null,
    private readonly ctx: PinContext,
  ) {}

  private highlightTarget(): { parentId: string | null; childId: string | null } {
    const id = this.highlightedId;
    if (!id) return { parentId: null, childId: null };
    const parentId = this.childToParent.get(id);
    return parentId ? { parentId, childId: id } : { parentId: id, childId: null };
  }

  private crowded(map: MapLibreMap, markers: MapMarker[]): Set<string> {
    return detectCrowdedMarkerIds((p) => map.project(p), markers, this.forceExpanded);
  }

  clear(): void {
    for (const marker of this.placed) marker.remove();
    this.placed = [];
    this.byId.clear();
  }

  render(next: MapMarker[]): void {
    const map = this.map();
    if (!map) return;
    if (this.rendering) {
      this.pending = next;
      queueMicrotask(() => {
        if (this.pending === next) this.render(this.pending);
      });
      return;
    }
    this.rendering = true;
    try {
      this.place(map, next);
    } finally {
      this.rendering = false;
    }
  }

  private place(map: MapLibreMap, next: MapMarker[]): void {
    this.clear();
    const stacked = stackOverlappingCoordinates(next);
    this.forceExpanded = stacked.forceExpandedIds;
    this.childToParent = stacked.childToParentId;
    const crowded = this.crowded(map, stacked.markers);
    this.lastCrowdedKey = [...crowded].sort().join(',');
    const highlight = this.highlightTarget();
    stacked.markers.forEach((item, stackIndex) => {
      const highlighted = item.id === highlight.parentId;
      const forced = this.forceExpanded.has(item.id);
      const compact = crowded.has(item.id) && !highlighted && !forced;
      const element = buildMarkerElement(
        item,
        compact,
        highlighted,
        highlighted ? highlight.childId : null,
        this.ctx,
      );
      if (forced || highlighted) element.style.zIndex = String(40 + stackIndex);
      const marker = new Marker({ element, anchor: compact && !highlighted ? 'center' : 'bottom' })
        .setLngLat([item.longitude, item.latitude])
        .addTo(map);
      this.placed.push(marker);
      this.byId.set(item.id, { marker, item });
      for (const child of item.children ?? []) this.byId.set(child.id, { marker, item });
    });
  }

  /** After a zoom, re-render only when which pins are crowded changed. */
  scheduleCompactRefresh(): void {
    if (this.compactDebounce) clearTimeout(this.compactDebounce);
    this.compactDebounce = setTimeout(() => {
      const map = this.map();
      if (!map || this.pending.length === 0 || this.rendering) return;
      const stacked = stackOverlappingCoordinates(this.pending);
      this.forceExpanded = stacked.forceExpandedIds;
      this.childToParent = stacked.childToParentId;
      const key = [...this.crowded(map, stacked.markers)].sort().join(',');
      if (key !== this.lastCrowdedKey) this.render(this.pending);
    }, 400);
  }

  highlight(markerId: string | null): void {
    this.highlightedId = markerId;
    if (this.map() && this.pending.length > 0) this.render(this.pending);
  }

  position(markerId: string): { latitude: number; longitude: number } | null {
    const entry = this.byId.get(markerId);
    return entry ? { latitude: entry.item.latitude, longitude: entry.item.longitude } : null;
  }

  destroy(): void {
    if (this.compactDebounce) clearTimeout(this.compactDebounce);
    this.clear();
    this.pending = [];
  }
}
