/**
 * web's `stores/mapFocus.ts`: a place another page wants shown on the map. A page calls
 * `openPlaceOnMap`; the shell opens the map panel and the panel centres on it and clears it,
 * so no feature imports another (ADR 0085).
 */
export interface MapFocusRequest {
  label: string;
  /** The place's coordinates when the page has them (web looks its location row up). */
  latitude: number | null;
  longitude: number | null;
  /** The page whose pin to select once the map is there (`/projects/<id>`, …). */
  select: string | null;
  /** Open the map full screen on it (a place tapped on a card or page; owner, 2026-11-23). */
  full: boolean;
  token: number;
}

export const mapFocus = $state<{ request: MapFocusRequest | null; open: boolean }>({
  request: null,
  open: false,
});

export function openPlaceOnMap(
  label: string,
  coords: { latitude: number; longitude: number } | null = null,
  select: string | null = null,
  full = true,
): void {
  const trimmed = label.trim();
  if (!trimmed) return;
  mapFocus.request = {
    label: trimmed,
    latitude: coords?.latitude ?? null,
    longitude: coords?.longitude ?? null,
    select,
    full,
    token: Date.now(),
  };
}

export function clearMapFocusRequest(): void {
  mapFocus.request = null;
}
