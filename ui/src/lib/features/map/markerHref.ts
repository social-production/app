import type { ResolvedPathname } from '$app/types';
import { resolve } from '$app/paths';
import { markerRoute, type MapMarkerItem } from '#lib/domain/mapMarkers.ts';

/** A map item's page, with web's `?activity=` (the path is resolved; the query rides along). */
export function markerHref(item: MapMarkerItem): ResolvedPathname {
  const route = markerRoute(item.href);
  if (!route) return resolve('/');
  const path =
    route.path === 'events'
      ? resolve('/events/[id]', { id: route.id })
      : route.path === 'help-requests'
        ? resolve('/help-requests/[id]', { id: route.id })
        : resolve('/projects/[id]', { id: route.id });
  return `${path}${route.query}` as ResolvedPathname;
}
