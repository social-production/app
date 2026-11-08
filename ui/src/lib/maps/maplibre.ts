/** web's MapLibre adapter (`maps/maplibre.ts` `createMapLibreAdapter`) as a class: camera,
 * radius circle, centre pin and viewport events; pins live in `Pins`, the start in `load.ts`. */
import { Marker, type GeoJSONSource, type Map as MapLibreMap } from 'maplibre-gl';
import { circlePolygon, radiusBounds, viewportRadiusKm } from './geo.ts';
import { loadMap } from './load.ts';
import { Pins } from './pins.ts';
import { FloatingTooltip } from './tooltip.ts';
import type { MapAdapter, MapMarker, MapViewport, MapViewportChange } from './types.ts';

const RADIUS_SOURCE_ID = 'sp-radius-circle';
const EMPTY = { type: 'FeatureCollection' as const, features: [] };

export class MapLibreAdapter implements MapAdapter {
  private map: MapLibreMap | null = null;
  private readonly tooltip = new FloatingTooltip();
  private readonly pins: Pins;
  private resizeObserver: ResizeObserver | null = null;
  private onError: ((message: string) => void) | null = null;
  private viewportHandler: ((change: MapViewportChange) => void) | null = null;
  private viewportDebounce: ReturnType<typeof setTimeout> | null = null;
  private suppress = false;
  private userGesture = false;
  private centerMarker: Marker | null = null;
  private radiusReady = false;

  constructor(navigate: (href: string) => void) {
    this.pins = new Pins(() => this.map, { tooltip: this.tooltip, navigate });
  }

  async mount(container: HTMLElement, viewport: MapViewport): Promise<void> {
    if (this.map) {
      this.resize();
      this.pins.render(this.pins.pending);
      return;
    }
    const { map, ready } = loadMap(container, viewport, (m) => this.onError?.(m));
    this.map = map;
    this.resizeObserver = new ResizeObserver(() => this.resize());
    this.resizeObserver.observe(container);
    try {
      await ready;
    } catch {
      return; // reported through onError; the partly loaded map stays
    }
    this.ensureRadiusLayers();
    this.attachViewportListeners(map);
    this.pins.render(this.pins.pending);
    this.resize();
  }

  private ensureRadiusLayers(): void {
    const map = this.map;
    if (!map || this.radiusReady) return;
    if (!map.getSource(RADIUS_SOURCE_ID)) {
      map.addSource(RADIUS_SOURCE_ID, { type: 'geojson', data: EMPTY });
      map.addLayer({
        id: 'sp-radius-fill',
        type: 'fill',
        source: RADIUS_SOURCE_ID,
        paint: { 'fill-color': '#2563eb', 'fill-opacity': 0.08 },
      });
      map.addLayer({
        id: 'sp-radius-line',
        type: 'line',
        source: RADIUS_SOURCE_ID,
        paint: { 'line-color': '#2563eb', 'line-opacity': 0.35, 'line-width': 2 },
      });
    }
    this.radiusReady = true;
  }

  private whenStyleReady(task: () => void): void {
    if (!this.map) return;
    if (this.map.isStyleLoaded()) task();
    else this.map.once('load', task);
  }

  /** Camera moves the map makes itself don't count as the person's (web's suppression). */
  private quietly(ms: number, done?: () => void): void {
    this.suppress = true;
    window.setTimeout(() => {
      this.suppress = false;
      done?.();
    }, ms);
  }

  private attachViewportListeners(map: MapLibreMap): void {
    const mark = (event?: { originalEvent?: unknown }): void => {
      if (!this.suppress && event?.originalEvent) this.userGesture = true;
    };
    map.on('movestart', mark);
    map.on('zoomstart', mark);
    map.on('moveend', () => this.scheduleViewportChange());
    map.on('zoomend', () => {
      this.scheduleViewportChange();
      this.pins.scheduleCompactRefresh();
    });
  }

  private scheduleViewportChange(): void {
    if (this.suppress) return;
    if (this.viewportDebounce) clearTimeout(this.viewportDebounce);
    this.viewportDebounce = setTimeout(() => this.emitViewportChange(), 400);
  }

  private emitViewportChange(): void {
    const radius = this.getViewportRadiusKm();
    if (!this.map || this.suppress || !this.viewportHandler || radius == null) return;
    const center = this.map.getCenter();
    const userInitiated = this.userGesture;
    this.userGesture = false;
    this.viewportHandler({
      center: { latitude: center.lat, longitude: center.lng },
      radiusKm: radius,
      userInitiated,
    });
  }

  setMarkers(markers: MapMarker[]): void {
    this.pins.pending = markers;
    if (!this.map) return;
    this.whenStyleReady(() => {
      this.ensureRadiusLayers();
      this.pins.render(this.pins.pending);
    });
  }

  setHighlightedMarker(markerId: string | null): void {
    this.pins.highlight(markerId);
  }

  setViewport(viewport: MapViewport): void {
    this.suppress = true;
    this.map?.jumpTo({
      center: [viewport.center.longitude, viewport.center.latitude],
      zoom: viewport.zoom,
    });
    this.resize();
    this.quietly(250);
  }

  fitToRadius(
    center: { latitude: number; longitude: number },
    radiusKm: number,
    onComplete?: () => void,
  ): void {
    const map = this.map;
    if (!map) return onComplete?.();
    this.suppress = true;
    map.fitBounds(radiusBounds(center.latitude, center.longitude, radiusKm), {
      padding: 28,
      maxZoom: radiusKm <= 25 ? 13 : radiusKm <= 100 ? 11 : radiusKm <= 500 ? 9 : 7,
      duration: 350,
    });
    map.once('moveend', () => this.quietly(300, onComplete));
  }

  getViewportRadiusKm(): number | null {
    if (!this.map) return null;
    const center = this.map.getCenter();
    const b = this.map.getBounds();
    return viewportRadiusKm(
      center.lat,
      center.lng,
      { latitude: b.getNorth(), longitude: b.getEast() },
      { latitude: b.getSouth(), longitude: b.getWest() },
    );
  }

  setRadiusCircle(center: { latitude: number; longitude: number }, radiusKm: number): void {
    this.whenStyleReady(() => {
      this.ensureRadiusLayers();
      const source = this.map?.getSource(RADIUS_SOURCE_ID) as GeoJSONSource | undefined;
      source?.setData({
        type: 'Feature',
        geometry: circlePolygon(center.latitude, center.longitude, radiusKm),
        properties: {},
      });
    });
  }

  clearRadiusCircle(): void {
    (this.map?.getSource(RADIUS_SOURCE_ID) as GeoJSONSource | undefined)?.setData(EMPTY);
  }

  setCenterMarker(center: { latitude: number; longitude: number } | null): void {
    this.centerMarker?.remove();
    this.centerMarker = null;
    if (!this.map || !center) return;
    const element = document.createElement('div');
    element.className = 'sp-map-center-marker';
    element.setAttribute('aria-hidden', 'true');
    this.centerMarker = new Marker({ element, anchor: 'bottom' })
      .setLngLat([center.longitude, center.latitude])
      .addTo(this.map);
  }

  centerOnMarker(markerId: string): void {
    const at = this.pins.position(markerId);
    if (!this.map || !at) return;
    this.suppress = true;
    this.map.panTo([at.longitude, at.latitude], { duration: 280 });
    this.map.once('moveend', () => this.quietly(200));
  }

  onViewportChange(handler: ((change: MapViewportChange) => void) | null): void {
    this.viewportHandler = handler;
  }

  resize(): void {
    const map = this.map;
    if (!map) return;
    this.suppress = true;
    map.resize();
    map.once('idle', () => {
      map.resize();
      this.quietly(300);
    });
  }

  setErrorHandler(handler: ((message: string) => void) | null): void {
    this.onError = handler;
  }

  destroy(): void {
    if (this.viewportDebounce) clearTimeout(this.viewportDebounce);
    this.tooltip.remove();
    this.resizeObserver?.disconnect();
    this.resizeObserver = null;
    this.pins.destroy();
    this.centerMarker?.remove();
    this.centerMarker = null;
    this.clearRadiusCircle();
    this.map?.remove();
    this.map = null;
    this.radiusReady = false;
    this.onError = null;
    this.viewportHandler = null;
  }
}
