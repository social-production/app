/** web's map filters (`location/mapPreferences.ts`), kept per device like the default location
 * (ADR 0081, 0085), under web's key with the agent key for web's user id. */

export interface MapFilterPreferences {
  radiusKm: string;
  windowFilter: string;
  entityFilter: string;
  upcomingOnly: boolean;
  updatedAt: string;
}

export const mapPreferencesKey = (me: string | null): string =>
  me ? `sp_map_preferences_${me}` : 'sp_map_preferences_anon';

export function parseMapPreferences(raw: string | null): MapFilterPreferences | null {
  if (!raw) return null;
  try {
    const p = JSON.parse(raw) as Partial<MapFilterPreferences>;
    if (typeof p.radiusKm !== 'string' || !p.radiusKm.trim()) return null;
    return {
      radiusKm: p.radiusKm.trim(),
      windowFilter: typeof p.windowFilter === 'string' ? p.windowFilter : 'all',
      entityFilter: typeof p.entityFilter === 'string' ? p.entityFilter : 'all',
      upcomingOnly: p.upcomingOnly !== false,
      updatedAt: typeof p.updatedAt === 'string' ? p.updatedAt : new Date().toISOString(),
    };
  } catch {
    return null;
  }
}

/** The stored shape, stamped now (web's `writeMapPreferences`). */
export const serializeMapPreferences = (p: Omit<MapFilterPreferences, 'updatedAt'>): string =>
  JSON.stringify({ ...p, updatedAt: new Date().toISOString() });
