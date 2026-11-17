/** The pin layer (Map Redesign, owner 2026-11-17): one round pin per thing; pins closer than
 * a pin's width merge into a count that zooms in when tapped (they used to shrink to dots);
 * the selected pin stays out of any count. web's same-spot stacking still applies first. */
import { Marker, type Map as MapLibreMap } from 'maplibre-gl';
import { buildClusterElement, buildMarkerElement, type PinContext } from './markerElement.ts';
import { stackOverlappingCoordinates } from './stacking.ts';
import type { MapMarker } from './types.ts';

const CLUSTER_PX = 42;

interface Cluster {
  members: MapMarker[];
  x: number;
  y: number;
}

/** Greedy screen-space clusters: each pin joins the first cluster within CLUSTER_PX. */
function clusterOf(map: MapLibreMap, markers: MapMarker[], keep: string | null): Cluster[] {
  const clusters: Cluster[] = [];
  for (const item of markers) {
    const p = map.project([item.longitude, item.latitude]);
    const near =
      item.id === keep
        ? undefined
        : clusters.find(
            (c) =>
              !c.members.some((m) => m.id === keep) &&
              (c.x - p.x) ** 2 + (c.y - p.y) ** 2 <= CLUSTER_PX ** 2,
          );
    if (near) near.members.push(item);
    else clusters.push({ members: [item], x: p.x, y: p.y });
  }
  return clusters;
}

const signature = (clusters: Cluster[]): string =>
  clusters
    .map((c) =>
      c.members
        .map((m) => m.id)
        .sort()
        .join('+'),
    )
    .sort()
    .join(',');

export class Pins {
  pending: MapMarker[] = [];
  private placed: Marker[] = [];
  private lastKey = '';
  private refresh: ReturnType<typeof setTimeout> | null = null;
  private highlightedId: string | null = null;
  private byId = new Map<string, MapMarker>();
  private childToParent = new Map<string, string>();

  constructor(
    private readonly map: () => MapLibreMap | null,
    private readonly ctx: PinContext,
  ) {}

  /** The pin a highlighted id shows on (an activity shows on its parent's pin). */
  private selectedPin(): string | null {
    const id = this.highlightedId;
    return id ? (this.childToParent.get(id) ?? id) : null;
  }

  clear(): void {
    for (const marker of this.placed) marker.remove();
    this.placed = [];
    this.byId.clear();
  }

  render(next: MapMarker[]): void {
    const map = this.map();
    this.pending = next;
    if (!map) return;
    this.clear();
    const stacked = stackOverlappingCoordinates(next);
    this.childToParent = stacked.childToParentId;
    const selected = this.selectedPin();
    const clusters = clusterOf(map, stacked.markers, selected);
    this.lastKey = signature(clusters);
    for (const item of stacked.markers) {
      this.byId.set(item.id, item);
      for (const child of item.children ?? []) this.byId.set(child.id, item);
    }
    for (const cluster of clusters) {
      const [only] = cluster.members;
      if (cluster.members.length === 1 && only) this.placePin(map, only, selected);
      else this.placeCluster(map, cluster.members);
    }
  }

  private placePin(map: MapLibreMap, item: MapMarker, selected: string | null): void {
    const highlighted = item.id === selected;
    const element = buildMarkerElement(item, highlighted, this.ctx);
    if (highlighted) element.style.zIndex = '50';
    const marker = new Marker({ element, anchor: 'bottom' })
      .setLngLat([item.longitude, item.latitude])
      .addTo(map);
    this.placed.push(marker);
  }

  private placeCluster(map: MapLibreMap, members: MapMarker[]): void {
    const lng = members.reduce((sum, m) => sum + m.longitude, 0) / members.length;
    const lat = members.reduce((sum, m) => sum + m.latitude, 0) / members.length;
    const element = buildClusterElement(members.length, () => {
      const lngs = members.map((m) => m.longitude);
      const lats = members.map((m) => m.latitude);
      const span = Math.max(
        Math.max(...lngs) - Math.min(...lngs),
        Math.max(...lats) - Math.min(...lats),
      );
      if (span < 0.0005) map.easeTo({ center: [lng, lat], zoom: Math.min(map.getZoom() + 3, 18) });
      else
        map.fitBounds(
          [
            [Math.min(...lngs), Math.min(...lats)],
            [Math.max(...lngs), Math.max(...lats)],
          ],
          { padding: 80, maxZoom: 18 },
        );
    });
    const marker = new Marker({ element, anchor: 'center' }).setLngLat([lng, lat]).addTo(map);
    this.placed.push(marker);
  }

  /** After a zoom, re-render only when the clusters changed. */
  scheduleCompactRefresh(): void {
    if (this.refresh) clearTimeout(this.refresh);
    this.refresh = setTimeout(() => {
      const map = this.map();
      if (!map || this.pending.length === 0) return;
      const stacked = stackOverlappingCoordinates(this.pending);
      const key = signature(clusterOf(map, stacked.markers, this.selectedPin()));
      if (key !== this.lastKey) this.render(this.pending);
    }, 250);
  }

  highlight(markerId: string | null): void {
    this.highlightedId = markerId;
    if (this.map() && this.pending.length > 0) this.render(this.pending);
  }

  position(markerId: string): { latitude: number; longitude: number } | null {
    const item = this.byId.get(markerId);
    return item ? { latitude: item.latitude, longitude: item.longitude } : null;
  }

  destroy(): void {
    if (this.refresh) clearTimeout(this.refresh);
    this.clear();
    this.pending = [];
  }
}
