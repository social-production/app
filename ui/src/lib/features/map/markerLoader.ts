/**
 * web's MapPanel data half: markers for the anchor and filters (`loadMarkers`, with one
 * broadening when a saved narrow filter finds nothing), the anchor place (`applyAnchor`), the
 * first centre (saved default, IP hint, device, Worldwide) and "show on map" requests.
 */
import { tick } from 'svelte';
import { devicePositionMessage } from '#lib/domain/defaultLocation.ts';
import { bySchedule, pinOf } from '#lib/domain/mapMarkers.ts';
import { GLOBAL_RADIUS_VALUE, effectiveRadiusKm } from '#lib/domain/mapRadius.ts';
import { emptyPlaceValue, type PlaceValue } from '#lib/domain/places.ts';
import { ipPlaceHint, devicePlace, searchPlaces } from '#lib/services/locations.ts';
import { getMapMarkers } from '#lib/services/map.ts';
import { readDefaultLocation, writeDefaultLocation } from '#lib/stores/defaultLocation.svelte.ts';
import type { MapFocusRequest } from '#lib/stores/mapFocus.svelte.ts';
import { DEFAULT_CAMERA_CENTER, type MapPanelState } from './panelState.svelte.ts';
import type { MapView } from './mapView.ts';

type Coords = { lat: number; lon: number; radiusKm?: number };

export class MarkerLoader {
  private token = 0;
  private debounce: ReturnType<typeof setTimeout> | null = null;

  constructor(
    private readonly s: MapPanelState,
    private readonly view: MapView,
    private readonly container: () => HTMLElement | undefined,
  ) {}

  schedule(coords?: Coords): void {
    if (!this.view.active) return;
    if (this.debounce) clearTimeout(this.debounce);
    this.debounce = setTimeout(() => void this.load(coords), 180);
  }

  private radiusFor(coords?: Coords): number {
    const v = this.view;
    if (coords?.radiusKm != null) return coords.radiusKm;
    if (v.viewportPanActive && v.viewportRadiusKm != null) return v.viewportRadiusKm;
    return v.currentRadiusKm ?? effectiveRadiusKm(this.s.radiusKm);
  }

  async load(coords?: Coords): Promise<void> {
    const s = this.s;
    const lat = coords?.lat ?? s.centerLat;
    const lon = coords?.lon ?? s.centerLon;
    const token = ++this.token;
    await this.view.mountIfNeeded(
      this.container(),
      lat != null && lon != null ? { lat, lon } : undefined,
    );
    if (lat == null || lon == null) {
      s.markers = [];
      s.listItems = [];
      return this.view.applyMarkers(false);
    }
    if (s.listItems.length === 0) s.loading = true;
    s.errorMessage = '';
    try {
      const items = await getMapMarkers({
        lat,
        lon,
        radiusKm: this.radiusFor(coords),
        window: s.windowFilter,
        filter: s.entityFilter,
        ...(s.showCustomDates && s.dateFrom ? { dateFrom: s.dateFrom } : {}),
        ...(s.showCustomDates && s.dateTo ? { dateTo: s.dateTo } : {}),
        upcomingOnly: s.upcomingOnly,
      });
      if (token !== this.token) return;
      s.markers = items.map(pinOf);
      s.listItems = bySchedule(items);
      await this.view.applyMarkers(true);
      this.view.syncRadiusOverlay();
    } catch (error) {
      if (token === this.token) {
        s.errorMessage = error instanceof Error ? error.message : 'Could not load map markers';
      }
    } finally {
      if (token === this.token) s.loading = false;
    }
  }

  /** First paint under a narrow saved filter came back empty: broaden once. */
  async loadBroadening(coords: Coords): Promise<void> {
    const s = this.s;
    await this.load(coords);
    if (s.listItems.length > 0) return;
    const narrow = ['today', 'week', 'month'].includes(s.windowFilter);
    if (!narrow && s.entityFilter === 'all') return;
    s.windowFilter = 'all';
    s.entityFilter = 'all';
    s.persistFilters();
    await this.load(coords);
  }

  private persistCenter(device?: boolean): void {
    const previous = readDefaultLocation(this.s.me);
    writeDefaultLocation(this.s.me, this.s.place, device ?? previous?.deviceGeolocation ?? false);
  }

  /** web's `applyAnchor`: one place drives the label, pin, radius, camera and markers. */
  async applyAnchor(
    next: PlaceValue,
    options: { device?: boolean; broadenEmpty?: boolean; persist?: boolean } = {},
  ): Promise<void> {
    const persist = options.persist !== false;
    this.s.place = next;
    if (persist) this.persistCenter(options.device);
    if (next.latitude == null || next.longitude == null) return;
    const at = { lat: next.latitude, lon: next.longitude };
    this.view.resetViewportQuery();
    this.s.fullscreenSearchOpen = false;
    await this.view.mountIfNeeded(this.container(), at);
    await tick();
    this.view.currentRadiusKm = effectiveRadiusKm(this.s.radiusKm);
    this.view.syncRadiusOverlay(at);
    await this.view.syncRadiusTo(at.lat, at.lon);
    this.view.currentRadiusKm = effectiveRadiusKm(this.s.radiusKm);
    this.view.syncRadiusOverlay(at);
    await (options.broadenEmpty ? this.loadBroadening(at) : this.load(at));
  }

  /** Centre on a place another page asked for, without changing the saved home area. */
  async focus(request: MapFocusRequest, onMissing: (label: string) => void): Promise<void> {
    const place: PlaceValue | null =
      request.latitude != null && request.longitude != null
        ? { ...emptyPlaceValue(), latitude: request.latitude, longitude: request.longitude }
        : ((await searchPlaces(request.label, 1).catch(() => []))[0] ?? null);
    if (place?.latitude == null || place.longitude == null) return onMissing(request.label);
    // A global radius would leave the camera on the whole world; look around the place.
    if (this.s.radiusKm === GLOBAL_RADIUS_VALUE) this.s.radiusKm = '10';
    await this.applyAnchor({ ...place, displayLabel: request.label }, { persist: false });
  }

  async useDevice(): Promise<void> {
    this.s.clearToast();
    const result = await devicePlace();
    if (!result.ok) return this.s.showToast(devicePositionMessage(result.error));
    await this.applyAnchor(result.place, { device: true, broadenEmpty: true });
  }

  /** web's `resolveInitialCenter`. */
  async initialCenter(): Promise<void> {
    const s = this.s;
    const saved = readDefaultLocation(s.me);
    if (saved?.place.latitude != null && saved.place.longitude != null) {
      s.place = saved.place;
      return;
    }
    const [hint] = await ipPlaceHint().catch(() => []);
    if (hint?.latitude != null && hint.longitude != null) {
      s.place = hint;
      return this.persistCenter();
    }
    if (saved?.deviceGeolocation) {
      const device = await devicePlace();
      if (device.ok) {
        s.place = device.place;
        return this.persistCenter(true);
      }
    }
    s.place = { ...emptyPlaceValue(), displayLabel: 'Worldwide', ...DEFAULT_CAMERA_CENTER };
    s.radiusKm = GLOBAL_RADIUS_VALUE;
  }

  destroy(): void {
    if (this.debounce) clearTimeout(this.debounce);
  }
}
