import { effectiveRadiusKm, GLOBAL_RADIUS_VALUE } from '#lib/domain/mapRadius.ts';

/** Owner's fix 3d (2026-11-17): the Region feed's distance, chosen in Settings › Regional and
 * kept on this device; the feed starts from it. */
const KEY = 'sp_region_radius';
export const DEFAULT_REGION_RADIUS = '25';

export function readRegionRadius(): string {
  try {
    const saved = localStorage.getItem(KEY);
    if (saved === GLOBAL_RADIUS_VALUE) return saved;
    return saved && /^\d+$/.test(saved) ? String(effectiveRadiusKm(saved)) : DEFAULT_REGION_RADIUS;
  } catch {
    return DEFAULT_REGION_RADIUS;
  }
}

export function writeRegionRadius(value: string): void {
  try {
    localStorage.setItem(KEY, value);
  } catch {
    // Private windows: the choice lasts for this page only.
  }
}
