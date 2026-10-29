import { gatewayApi } from './gateway.ts';
import { homeGateway } from './nodes.ts';

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
