import { deviceLocations } from './device.ts';
import { gatewayApi } from './gateway.ts';
import { homeGateway, inNativeApp } from './nodes.ts';

/** The public project feed from the home node's index (GET /index/projects). */
export function projectFeed<T>(query: Record<string, string>): Promise<T> {
  const gateway = homeGateway();
  if (!gateway) return Promise.reject(new Error('No node configured (VITE_SP_GATEWAYS).'));
  return gatewayApi.projects<T>(gateway, query);
}

/** People whose name or bio matches, from the home node's index (GET /index/people). */
export function peopleIndex<T>(query: Record<string, string>): Promise<T> {
  const gateway = homeGateway();
  if (!gateway) return Promise.reject(new Error('No node configured (VITE_SP_GATEWAYS).'));
  return gatewayApi.people<T>(gateway, query);
}

/** Channels and communities, first handle first (GET /index/scopes, ADR 0068). */
export function scopeIndex<T>(query: Record<string, string>): Promise<T> {
  const gateway = homeGateway();
  if (!gateway) return Promise.reject(new Error('No node configured (VITE_SP_GATEWAYS).'));
  return gatewayApi.scopes<T>(gateway, query);
}

/** Agents a bot review paused, from the home node's index (GET /index/paused, ADR 0074). */
export function pausedIndex(): Promise<string[]> {
  const gateway = homeGateway();
  if (!gateway) return Promise.reject(new Error('No node configured (VITE_SP_GATEWAYS).'));
  return gatewayApi.paused(gateway);
}

/** web's place routes on the home node's gateway (GET /locations/*, ADR 0079). The native
 * app runs them on the device instead, so places work with no gateway (ADR 0093). */
export function locationLookup<T>(
  route: 'search' | 'reverse' | 'ip-hint',
  query: Record<string, string>,
): Promise<T> {
  if (inNativeApp()) return deviceLocations<T>(route, query);
  const gateway = homeGateway();
  if (!gateway) return Promise.reject(new Error('No node configured (VITE_SP_GATEWAYS).'));
  return gatewayApi.locations<T>(gateway, route, query);
}

/** web's `/feeds/map-markers` and `/feeds/region` from the home node's index (ADR 0082–0084). */
export function placeIndex<T>(
  route: 'map-markers' | 'region',
  query: Record<string, string>,
): Promise<T> {
  const gateway = homeGateway();
  if (!gateway) return Promise.reject(new Error('No node configured (VITE_SP_GATEWAYS).'));
  return gatewayApi.places<T>(gateway, route, query);
}
