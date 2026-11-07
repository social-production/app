import {
  fromGateway,
  type GatewayLocation,
  type PlaceSearch,
  type PlaceValue,
} from '#lib/domain/places.ts';
import { GatewayError, locationLookup } from '#lib/holo/index.ts';

/** web's `services/queries/locations.ts` against the gateway's routes (ADR 0079). */

interface Items {
  items?: GatewayLocation[];
}

export async function searchPlaces(
  query: string,
  limit = 5,
  options: { countryCodes?: string | null; viewbox?: string | null } = {},
): Promise<PlaceValue[]> {
  const params: Record<string, string> = { q: query, limit: String(limit) };
  if (options.countryCodes?.trim()) params.country_codes = options.countryCodes.trim();
  if (options.viewbox?.trim()) params.viewbox = options.viewbox.trim();
  const res = await locationLookup<Items>('search', params);
  return (res.items ?? []).map(fromGateway);
}

export async function reversePlace(latitude: number, longitude: number): Promise<PlaceValue[]> {
  const res = await locationLookup<Items>('reverse', {
    lat: String(latitude),
    lon: String(longitude),
  });
  return (res.items ?? []).map(fromGateway);
}

export async function ipPlaceHint(): Promise<PlaceValue[]> {
  const res = await locationLookup<Items>('ip-hint', {});
  return (res.items ?? []).map(fromGateway);
}

/** web's picker message: a rate limit reads differently from an outage. */
export function searchFailure(error: unknown): string {
  return error instanceof GatewayError && error.status === 429
    ? 'Too many searches. Wait a moment and try again.'
    : 'Could not search places right now.';
}

/** What the picker needs (`PlaceSearch`). */
export const placeSearch: PlaceSearch = { search: searchPlaces, failure: searchFailure };
