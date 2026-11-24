/**
 * The map panel's state (web's `MapPanel.svelte` variables): the anchor place, filters, the
 * results, and the layout flags. `MapView` drives the map from it, `MarkerLoader` fills it.
 */
import { t } from '#lib/i18n/index.ts';
import { emptyPlaceValue, type PlaceValue } from '#lib/domain/places.ts';
import type { MapMarkerItem } from '#lib/domain/mapMarkers.ts';
import { writeMapPreferences } from '#lib/stores/mapPreferences.svelte.ts';
import type { MapMarker } from '#lib/maps/adapter.ts';

/** Camera fallback when the place is Worldwide: land and seed pins, not empty ocean. */
export const DEFAULT_CAMERA_CENTER = { latitude: -37.8136, longitude: 144.9631 };
export const WORLD_ZOOM = 2;
export const LOCAL_ZOOM = 11;

export const windowOptions = [
  { value: 'today', label: t('Today') },
  { value: 'week', label: t('This week') },
  { value: 'month', label: t('This month') },
  { value: 'all', label: t('All time') },
  { value: 'custom', label: t('Custom range') },
];
export const filterOptions = [
  { value: 'all', label: t('All') },
  { value: 'events', label: t('Events') },
  { value: 'meetups', label: t('Meetups') },
  { value: 'projects', label: t('Projects') },
  { value: 'help_requests', label: t('Help requests') },
];

export class MapPanelState {
  me: string | null = null;
  place = $state<PlaceValue>(emptyPlaceValue());
  radiusKm = $state('25');
  upcomingOnly = $state(true);
  windowFilter = $state('all');
  entityFilter = $state('all');
  dateFrom = $state('');
  dateTo = $state('');
  // The map opens loading, so the list shows skeletons rather than "Nothing here" at first.
  loading = $state(true);
  errorMessage = $state('');
  locationToast = $state('');
  mapTileError = $state('');
  markers: MapMarker[] = [];
  listItems = $state<MapMarkerItem[]>([]);
  highlightedId = $state<string | null>(null);
  /** The pin whose card is open (Map Redesign). */
  selectedId = $state<string | null>(null);
  mapFullscreen = $state(false);
  isMobile = $state(false);
  isCompactMap = $state(false);
  fullscreenSearchOpen = $state(false);
  mapStageHeightPx = $state(0);
  private toastTimer: ReturnType<typeof setTimeout> | null = null;

  get centerLat(): number | null {
    return this.place.latitude;
  }
  get centerLon(): number | null {
    return this.place.longitude;
  }
  get hasCenter(): boolean {
    return this.centerLat != null && this.centerLon != null;
  }
  get showCustomDates(): boolean {
    return this.windowFilter === 'custom';
  }
  /** On phones and in fullscreen the filters sit over the map. */
  get overlayFilters(): boolean {
    return this.isCompactMap || this.mapFullscreen;
  }

  persistFilters(): void {
    writeMapPreferences(this.me, {
      radiusKm: this.radiusKm,
      windowFilter: this.windowFilter,
      entityFilter: this.entityFilter,
      upcomingOnly: this.upcomingOnly,
    });
  }

  showToast(message: string): void {
    this.locationToast = message;
    if (this.toastTimer) clearTimeout(this.toastTimer);
    this.toastTimer = setTimeout(() => {
      this.locationToast = '';
      this.toastTimer = null;
    }, 4000);
  }

  clearToast(): void {
    this.locationToast = '';
    if (this.toastTimer) clearTimeout(this.toastTimer);
    this.toastTimer = null;
  }

  /** web's `updateMapStageHeight`: phones give the map 45% of the screen. */
  updateStageHeight(): void {
    this.mapStageHeightPx = !this.isMobile || this.mapFullscreen ? 0 : window.innerHeight * 0.45;
  }

  destroy(): void {
    this.clearToast();
  }
}
