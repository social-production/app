/**
 * web's default location (`location/defaults.ts`, `geolocation.ts`): the place the Region feed
 * and the map start from, plus the device-location opt-in. Kept per device (ADR 0081).
 */
import { t } from '#lib/i18n/index.ts';
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
        displayLabel: label || (coords ? t('Current location') : ''),
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

/** web's `devicePositionErrorMessage`, in plainer words: what happened and what to do. */
export function devicePositionMessage(error: DevicePositionError): string {
  switch (error) {
    case 'denied':
      return t(
        "Your browser is blocking location for this site. Allow it in the browser's site settings, or search for a place.",
      );
    case 'timeout':
      return t('Finding your location took too long. Try again, or search for a place.');
    case 'unsupported':
      return t("This browser can't share your location. Search for a place instead.");
    case 'disabled':
      return t(
        'Location is turned off on this device. Turn it on in settings, or search for a place.',
      );
    default:
      return t("We couldn't find your location. Search for a place instead.");
  }
}
