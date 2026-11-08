/** web's map adapter boundary (`maps/adapter.ts`): MapLibre is an implementation detail;
 * tests may swap the factory. */
import { MapLibreAdapter } from './maplibre.ts';
import type { MapAdapter } from './types.ts';

export type { MapAdapter, MapMarker, MapViewport, MapViewportChange } from './types.ts';

type Factory = (navigate: (href: string) => void) => MapAdapter;

const browserAdapter: Factory = (navigate) => new MapLibreAdapter(navigate);
let factory: Factory = browserAdapter;

export function setMapAdapterFactory(next: Factory): void {
  factory = next;
}

export function resetMapAdapterFactory(): void {
  factory = browserAdapter;
}

export function createMapAdapter(navigate: (href: string) => void): MapAdapter {
  return factory(navigate);
}
