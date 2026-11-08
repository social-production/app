/** web's map start (`maps/maplibre.ts` `mount`): Carto's Positron style, falling back to
 * MapLibre's demo style when Carto can't be reached within 4.5 s or fails to load. */
import { Map as MapLibreMap, NavigationControl, setWorkerUrl } from 'maplibre-gl';
import workerUrl from 'maplibre-gl/dist/maplibre-gl-worker.mjs?worker&url';
import 'maplibre-gl/dist/maplibre-gl.css';
import type { MapViewport } from './types.ts';

// MapLibre 6's worker is its own ESM chunk; Vite needs an explicit URL or tiles never paint.
setWorkerUrl(workerUrl);

const DEFAULT_STYLE = 'https://basemaps.cartocdn.com/gl/positron-gl-style/style.json';
const FALLBACK_STYLE = 'https://demotiles.maplibre.org/style.json';
const FALLBACK_AFTER_MS = 4500;
export const CARTO_FAILURE =
  'Map tiles could not be loaded from the internet (Carto basemaps). Local auth and feeds can still work — check DNS, firewall, VPN, or ad-blockers if the rest of the app is fine.';
const CARTO_PATTERN = /networkerror|failed to fetch|ajaxerror|cartocdn|basemaps\.cartocdn/i;

/** Resolves once a style has loaded; reports a failure through `onError` and rejects. */
export function loadMap(
  container: HTMLElement,
  viewport: MapViewport,
  onError: (message: string) => void,
): { map: MapLibreMap; ready: Promise<void> } {
  const map = new MapLibreMap({
    container,
    style: DEFAULT_STYLE,
    center: [viewport.center.longitude, viewport.center.latitude],
    zoom: viewport.zoom,
  });
  map.addControl(new NavigationControl({ visualizePitch: false }), 'top-right');
  const ready = new Promise<void>((resolve, reject) => {
    let settled = false;
    let usedFallback = false;
    const finish = (error?: string): void => {
      if (settled) return;
      settled = true;
      window.clearTimeout(timer);
      if (error) {
        onError(error);
        reject(new Error(error));
      } else resolve();
    };
    const fallBack = (): boolean => {
      if (usedFallback) return false;
      usedFallback = true;
      window.clearTimeout(timer);
      try {
        map.setStyle(FALLBACK_STYLE);
        map.once('load', () => finish());
        return true;
      } catch {
        return false;
      }
    };
    const timer = window.setTimeout(() => {
      if (!settled && !fallBack()) finish('Map tiles could not be loaded.');
    }, FALLBACK_AFTER_MS);
    map.once('load', () => finish());
    map.on('error', (event) => {
      const raw = event.error?.message?.trim() || 'Map tiles could not be loaded.';
      const carto = CARTO_PATTERN.test(raw);
      if (carto && fallBack()) return;
      finish(carto ? CARTO_FAILURE : raw);
    });
  });
  return { map, ready };
}
