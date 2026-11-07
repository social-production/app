/**
 * web's default location (`location/defaults.ts`, `geolocation.ts`): the place the Region feed
 * and the map start from, plus the device-location opt-in. Kept per device (ADR 0081).
 */
import { emptyPlaceValue, type PlaceValue } from './places.ts';

export interface DefaultLocation {
  place: PlaceValue;
  /** web's `deviceGeolocationEnabled`: the map's "Use my location" may ask the device. */
  deviceGeolocation: boolean;
  updatedAt: string;
}

/** web's key, by the account's agent key instead of web's user id. */
export const defaultLocationKey = (me: string | null): string =>
  me ? `sp_default_location_${me}` : 'sp_default_location_anon';

const text = (v: unknown): string | null => (typeof v === 'string' ? v : null);

/** web's `parsePreference`: a label or coordinates, or only the device opt-in. */
export function parseDefaultLocation(raw: string | null): DefaultLocation | null {
  if (!raw) return null;
  try {
    const p = JSON.parse(raw) as Record<string, unknown>;
    const lat = p.latitude;
    const lon = p.longitude;
    const coords =
      typeof lat === 'number' && typeof lon === 'number' && isFinite(lat) && isFinite(lon);
    const label = text(p.displayLabel)?.trim() ?? '';
    const device = p.deviceGeolocationEnabled === true;
    if (!label && !coords && !device) return null;
    return {
      place: {
        ...emptyPlaceValue(),
        displayLabel: label || (coords ? 'Current location' : ''),
        latitude: coords ? lat : null,
        longitude: coords ? lon : null,
        region: text(p.region),
        country: text(p.country),
        precision: p.precision === 'exact' ? 'exact' : 'approximate',
        providerPlaceId: text(p.providerPlaceId),
      },
      deviceGeolocation: device,
      updatedAt: text(p.updatedAt) ?? new Date().toISOString(),
    };
  } catch {
    return null;
  }
}

/** The stored shape (web's field names, so a later account sync can read it unchanged). */
export function serializeDefaultLocation(place: PlaceValue, deviceGeolocation: boolean): string {
  return JSON.stringify({
    displayLabel: place.displayLabel.trim(),
    latitude: place.latitude,
    longitude: place.longitude,
    region: place.region,
    country: place.country,
    precision: place.precision,
    providerPlaceId: place.providerPlaceId,
    deviceGeolocationEnabled: deviceGeolocation,
    updatedAt: new Date().toISOString(),
  });
}

export type DevicePositionError = 'disabled' | 'unsupported' | 'denied' | 'timeout' | 'unavailable';

/** web's `devicePositionErrorMessage`. */
export function devicePositionMessage(error: DevicePositionError): string {
  switch (error) {
    case 'denied':
      return 'Location permission denied. Search for a place instead.';
    case 'timeout':
      return 'Location request timed out. Try again or search for a place.';
    case 'unsupported':
      return 'This browser does not support device location.';
    case 'disabled':
      return 'Device location is off. Enable it in settings or search for a place.';
    default:
      return 'Could not determine your location. Search for a place instead.';
  }
}
