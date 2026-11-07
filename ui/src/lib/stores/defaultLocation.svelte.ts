import {
  defaultLocationKey,
  parseDefaultLocation,
  serializeDefaultLocation,
  type DefaultLocation,
} from '#lib/domain/defaultLocation.ts';
import type { PlaceValue } from '#lib/domain/places.ts';

/** web's `readDefaultLocation` / `writeDefaultLocation`, in this device's storage (ADR 0081). */
export function readDefaultLocation(me: string | null): DefaultLocation | null {
  try {
    return parseDefaultLocation(localStorage.getItem(defaultLocationKey(me)));
  } catch {
    return null;
  }
}

export function writeDefaultLocation(
  me: string | null,
  place: PlaceValue,
  deviceGeolocation: boolean,
): void {
  try {
    localStorage.setItem(
      defaultLocationKey(me),
      serializeDefaultLocation(place, deviceGeolocation),
    );
  } catch {
    // Storage can be off (private windows); the setting then lasts for this page only.
  }
}
