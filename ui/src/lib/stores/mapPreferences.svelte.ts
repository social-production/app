import {
  mapPreferencesKey,
  parseMapPreferences,
  serializeMapPreferences,
  type MapFilterPreferences,
} from '#lib/domain/mapPreferences.ts';

/** web's `readMapPreferences` / `writeMapPreferences`, in this device's storage (ADR 0085). */
export function readMapPreferences(me: string | null): MapFilterPreferences | null {
  try {
    return parseMapPreferences(localStorage.getItem(mapPreferencesKey(me)));
  } catch {
    return null;
  }
}

export function writeMapPreferences(
  me: string | null,
  preferences: Omit<MapFilterPreferences, 'updatedAt'>,
): void {
  try {
    localStorage.setItem(mapPreferencesKey(me), serializeMapPreferences(preferences));
  } catch {
    // Storage can be off (private windows); the filters then last for this page only.
  }
}
