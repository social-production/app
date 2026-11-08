import { devicePositionMessage } from '#lib/domain/defaultLocation.ts';
import { effectiveRadiusKm, normalizeRadiusFromUrl } from '#lib/domain/mapRadius.ts';
import { emptyPlaceValue, type PlaceValue } from '#lib/domain/places.ts';
import type { FeedSort, FeedWindow } from '#lib/domain/projects.ts';
import type { RegionItem, RegionLink } from '#lib/domain/regionFeed.ts';
import { explain } from '#lib/services/errors.ts';
import { devicePlace, searchPlaces } from '#lib/services/locations.ts';
import { regionFeed } from '#lib/services/region.ts';
import { readDefaultLocation, writeDefaultLocation } from '#lib/stores/defaultLocation.svelte.ts';

/**
 * web's PublicFeed Region scope: a centre (your default place, a link's, a searched place or
 * the device), a radius, "Include online list", and pages of `/index/region` (ADR 0084).
 */
export class RegionFeedState {
  lat = $state<number | null>(null);
  lon = $state<number | null>(null);
  placeQuery = $state('');
  radius = $state('25');
  includeOnline = $state(false);
  suggestions = $state<PlaceValue[]>([]);
  message = $state('');
  items = $state<RegionItem[]>([]);
  total = $state(0);
  loading = $state(false);
  error = $state<string>();
  me: string | null = null;
  private searchTimer: ReturnType<typeof setTimeout> | null = null;

  readonly hasCenter = $derived(this.lat != null && this.lon != null);
  readonly hasMore = $derived(this.items.length < this.total);

  constructor(private readonly query: () => { sort: FeedSort; window: FeedWindow }) {}

  /** web's `hydrateRegionCenter`: your default place. */
  hydrate(me: string | null): void {
    this.me = me;
    const saved = readDefaultLocation(me);
    if (saved?.place.latitude != null && saved.place.longitude != null) {
      this.setCenter(saved.place.latitude, saved.place.longitude, saved.place.displayLabel);
    }
  }

  /** A link's place and radius (the map's "Open as list"), over the default. */
  follow(link: RegionLink): void {
    this.setCenter(link.lat, link.lon, link.place || this.placeQuery);
    this.radius = normalizeRadiusFromUrl(link.radius) ?? '25';
  }

  private setCenter(lat: number, lon: number, label: string): void {
    this.lat = lat;
    this.lon = lon;
    this.placeQuery = label;
  }

  async reload(): Promise<void> {
    this.items = [];
    this.total = 0;
    await this.more();
  }

  async more(): Promise<void> {
    if (this.lat == null || this.lon == null) return;
    this.loading = true;
    this.error = undefined;
    try {
      const { sort, window } = this.query();
      const page = await regionFeed({
        lat: this.lat,
        lon: this.lon,
        radiusKm: effectiveRadiusKm(this.radius),
        sort,
        window,
        filter: 'all',
        includeOnline: this.includeOnline,
        offset: this.items.filter((i) => !('isOnline' in i && i.isOnline)).length,
      });
      // Online items ride along after every page (web); keep each once.
      const seen = this.items.map((i) => i.id);
      this.items = [...this.items, ...page.items.filter((i) => !seen.includes(i.id))];
      this.total = page.total;
    } catch (e) {
      this.error = explain(e);
    } finally {
      this.loading = false;
    }
  }

  /** web's debounced place search under the toolbar. */
  search(query: string): void {
    this.placeQuery = query;
    if (this.searchTimer) clearTimeout(this.searchTimer);
    this.searchTimer = setTimeout(() => {
      if (!query.trim()) return void (this.suggestions = []);
      void searchPlaces(query, 5)
        .then((found) => (this.suggestions = found.filter((p) => p.latitude != null)))
        .catch(() => (this.suggestions = []));
    }, 250);
  }

  /** web's `selectPlace`: the place becomes the centre and your default location. */
  select(place: PlaceValue): Promise<void> {
    if (place.latitude == null || place.longitude == null) return Promise.resolve();
    this.setCenter(place.latitude, place.longitude, place.displayLabel);
    this.suggestions = [];
    writeDefaultLocation(this.me, place, false);
    return this.reload();
  }

  /** web's `useDeviceCenter`. */
  async useDevice(): Promise<void> {
    const result = await devicePlace();
    if (!result.ok) return void (this.message = devicePositionMessage(result.error));
    this.message = '';
    const place = result.place;
    this.setCenter(place.latitude ?? 0, place.longitude ?? 0, place.displayLabel);
    writeDefaultLocation(this.me, { ...emptyPlaceValue(), ...place }, true);
    await this.reload();
  }
}
