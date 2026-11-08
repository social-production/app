/**
 * web's MapPanel camera half: mounting the adapter, the centre pin and radius circle, fitting
 * the camera to the radius, and adopting a zoomed viewport's radius (`syncRadius*`,
 * `mountMapIfNeeded`, the viewport handler). `onViewportLoad` asks the loader for markers.
 */
import { tick } from 'svelte';
import {
  GLOBAL_RADIUS_VALUE,
  effectiveRadiusKm,
  shouldAdoptViewportRadius,
  viewportRadiusDisplayValue,
} from '#lib/domain/mapRadius.ts';
import { createMapAdapter, type MapAdapter, type MapViewportChange } from '#lib/maps/adapter.ts';
import { CARTO_FAILURE } from '#lib/maps/load.ts';
import {
  DEFAULT_CAMERA_CENTER,
  LOCAL_ZOOM,
  WORLD_ZOOM,
  type MapPanelState,
} from './panelState.svelte.ts';

const RADIUS_SYNC_BUFFER_MS = 300;
type Coords = { lat: number; lon: number; radiusKm?: number };

/** web's `explainMapTileError`. */
function explain(raw: string): string {
  const lower = raw.trim().toLowerCase();
  const network = ['networkerror', 'failed to fetch', 'ajaxerror', 'load failed', 'cartocdn'];
  return !lower || network.some((n) => lower.includes(n)) ? CARTO_FAILURE : raw.trim();
}

export class MapView {
  adapter: MapAdapter | null = null;
  mounted = false;
  active = true;
  syncingFromRadius = false;
  stageTransitioning = false;
  currentRadiusKm: number | null = null;
  viewportPanActive = false;
  viewportRadiusKm: number | null = null;
  private lastViewportKey = '';
  private suppressLoads = false;
  private timers = new Map<string, ReturnType<typeof setTimeout>>();

  constructor(
    private readonly s: MapPanelState,
    private readonly navigate: (href: string) => void,
    private readonly onViewportLoad: (coords: Coords) => void,
  ) {}

  private after(name: string, ms: number, task: () => void): void {
    const old = this.timers.get(name);
    if (old) clearTimeout(old);
    this.timers.set(
      name,
      setTimeout(() => {
        this.timers.delete(name);
        task();
      }, ms),
    );
  }

  pauseViewportLoads(ms = 400): void {
    this.suppressLoads = true;
    this.after('suppress', ms, () => (this.suppressLoads = false));
  }

  scheduleResize(ms = 50): void {
    if (!this.active) return;
    this.pauseViewportLoads(ms + 350);
    this.after('resize', ms, () => this.adapter?.resize?.());
  }

  resetViewportQuery(): void {
    this.viewportPanActive = false;
    this.viewportRadiusKm = null;
    this.lastViewportKey = '';
  }

  highlight(markerId: string | null, center = false): void {
    this.s.highlightedId = markerId;
    this.adapter?.setHighlightedMarker?.(markerId);
    if (center && markerId) this.adapter?.centerOnMarker?.(markerId);
  }

  syncRadiusOverlay(at?: { lat: number; lon: number }): void {
    const lat = at?.lat ?? this.s.centerLat;
    const lon = at?.lon ?? this.s.centerLon;
    if (!this.adapter || lat == null || lon == null) {
      this.adapter?.setCenterMarker?.(null);
      this.adapter?.clearRadiusCircle?.();
      return;
    }
    this.adapter.setCenterMarker?.({ latitude: lat, longitude: lon });
    const radius =
      this.currentRadiusKm ??
      this.adapter.getViewportRadiusKm?.() ??
      effectiveRadiusKm(this.s.radiusKm);
    this.adapter.setRadiusCircle?.({ latitude: lat, longitude: lon }, radius);
  }

  private endRadiusSync(): void {
    this.after('radius', RADIUS_SYNC_BUFFER_MS, () => {
      this.syncingFromRadius = false;
      this.currentRadiusKm = effectiveRadiusKm(this.s.radiusKm);
      this.syncRadiusOverlay();
    });
  }

  /** web's `syncRadiusToMapAt`: jump to the centre, draw pin and circle, fit the radius. */
  syncRadiusTo(lat: number, lon: number): Promise<void> {
    const adapter = this.adapter;
    if (!adapter) return Promise.resolve();
    this.syncingFromRadius = true;
    const global = this.s.radiusKm === GLOBAL_RADIUS_VALUE;
    const km = effectiveRadiusKm(this.s.radiusKm);
    const zoom = global ? WORLD_ZOOM : km <= 25 ? 12 : km <= 100 ? 10 : 8;
    const center = { latitude: lat, longitude: lon };
    adapter.setViewport({ center, zoom });
    adapter.setCenterMarker?.(center);
    if (global) adapter.clearRadiusCircle?.();
    else {
      this.currentRadiusKm = km;
      adapter.setRadiusCircle?.(center, km);
    }
    return new Promise((resolve) => {
      const done = (): void => {
        this.endRadiusSync();
        resolve();
      };
      if (global) done();
      else adapter.fitToRadius?.(center, km, done);
    });
  }

  syncRadiusToCenter(): Promise<void> {
    const { centerLat: lat, centerLon: lon } = this.s;
    return lat == null || lon == null ? Promise.resolve() : this.syncRadiusTo(lat, lon);
  }

  private onViewport(viewport: MapViewportChange): void {
    if (!this.active || this.syncingFromRadius || this.stageTransitioning || this.suppressLoads) {
      return;
    }
    const { latitude, longitude } = viewport.center;
    const key = `${latitude.toFixed(3)}:${longitude.toFixed(3)}:${Math.round(viewport.radiusKm)}`;
    if (key === this.lastViewportKey) return;
    this.lastViewportKey = key;
    this.viewportPanActive = true;
    this.viewportRadiusKm = viewport.radiusKm;
    const selected = effectiveRadiusKm(this.s.radiusKm);
    const adopt = shouldAdoptViewportRadius(
      selected,
      viewport.radiusKm,
      Boolean(viewport.userInitiated),
    );
    this.currentRadiusKm = adopt ? viewport.radiusKm : selected;
    this.syncRadiusOverlay();
    if (adopt && !this.syncingFromRadius) {
      const next = viewportRadiusDisplayValue(viewport.radiusKm);
      if (next !== this.s.radiusKm) {
        this.s.radiusKm = next;
        this.s.persistFilters();
      }
    }
    this.onViewportLoad({ lat: latitude, lon: longitude, radiusKm: this.currentRadiusKm });
  }

  async mountIfNeeded(container: HTMLElement | undefined, at?: { lat: number; lon: number }) {
    if (!container || this.mounted) return;
    const centered = at != null;
    const center = at ? { latitude: at.lat, longitude: at.lon } : DEFAULT_CAMERA_CENTER;
    this.adapter = createMapAdapter(this.navigate);
    this.adapter.setErrorHandler?.((message) => (this.s.mapTileError = explain(message)));
    this.adapter.onViewportChange?.((v) => this.onViewport(v));
    try {
      await this.adapter.mount(container, { center, zoom: centered ? LOCAL_ZOOM : WORLD_ZOOM });
      this.mounted = true;
    } catch (error) {
      this.s.mapTileError = explain(error instanceof Error ? error.message : '');
    }
  }

  /** web's `applyMarkersToMap`: set, then set again once the layout settles. */
  async applyMarkers(centered: boolean): Promise<void> {
    this.adapter?.setMarkers(centered ? this.s.markers : []);
    await tick();
    window.setTimeout(() => {
      this.adapter?.setMarkers(centered ? this.s.markers : []);
      this.syncRadiusOverlay();
    }, 120);
    this.pauseViewportLoads(350);
  }

  destroy(): void {
    for (const timer of this.timers.values()) clearTimeout(timer);
    this.timers.clear();
    this.adapter?.destroy();
    this.adapter = null;
    this.mounted = false;
  }
}
