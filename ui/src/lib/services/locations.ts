import type { DevicePositionError } from '#lib/domain/defaultLocation.ts';
import {
  emptyPlaceValue,
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

/** web's `requestDevicePosition`: one GPS fix (only when the person asks), named by a reverse
 * lookup, or "Current location" when that fails. */
export async function devicePlace(): Promise<
  { ok: true; place: PlaceValue } | { ok: false; error: DevicePositionError }
> {
  if (!('geolocation' in navigator)) return { ok: false, error: 'unsupported' };
  const fix = await new Promise<GeolocationPosition | DevicePositionError>((resolve) => {
    navigator.geolocation.getCurrentPosition(resolve, (e) =>
      resolve(e.code === 1 ? 'denied' : e.code === 3 ? 'timeout' : 'unavailable'),
    );
  });
  if (typeof fix === 'string') return { ok: false, error: fix };
  const { latitude, longitude } = fix.coords;
  const named = await reversePlace(latitude, longitude).catch(() => []);
  const first = named[0];
  return {
    ok: true,
    place: {
      ...emptyPlaceValue(),
      displayLabel: first?.displayLabel ?? 'Current location',
      latitude,
      longitude,
      region: first?.region ?? null,
      country: first?.country ?? null,
      providerPlaceId: first?.providerPlaceId ?? null,
    },
  };
}

/** web's `useIpLocation` messages. */
export function ipHintFailure(error: unknown): string {
  if (error instanceof GatewayError && error.status === 429)
    return 'Too many IP location requests. Wait a moment and try again.';
  if (error instanceof GatewayError && error.message.includes('ip_location_unavailable'))
    return 'IP location is unavailable on localhost. Use search or device location, or try again from a network address.';
  return 'IP location is unavailable right now.';
}
