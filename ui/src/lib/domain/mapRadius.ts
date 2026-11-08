/** web's map radius (`location/radius.ts`): presets, "Global", and when a zoom adopts a new one. */

export const GLOBAL_RADIUS_VALUE = 'global';
export const GLOBAL_RADIUS_KM = 20_000;

export const radiusPresetOptions = [
  { value: '10', label: '10 km' },
  { value: '25', label: '25 km' },
  { value: '50', label: '50 km' },
  { value: '100', label: '100 km' },
  { value: '250', label: '250 km' },
  { value: '500', label: '500 km' },
  { value: '1000', label: '1000 km' },
  { value: GLOBAL_RADIUS_VALUE, label: 'Global' },
];

export function effectiveRadiusKm(value: string, fallback = 25): number {
  if (value === GLOBAL_RADIUS_VALUE) return GLOBAL_RADIUS_KM;
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) return fallback;
  return Math.min(GLOBAL_RADIUS_KM, Math.max(1, Math.round(parsed)));
}

export function radiusForUrl(value: string): string {
  return value === GLOBAL_RADIUS_VALUE ? String(GLOBAL_RADIUS_KM) : value;
}

/** A live viewport radius for the combobox (exact km, not snapped to presets). */
export function viewportRadiusDisplayValue(radiusKm: number): string {
  if (!Number.isFinite(radiusKm) || radiusKm >= GLOBAL_RADIUS_KM * 0.6) return GLOBAL_RADIUS_VALUE;
  return String(Math.max(1, Math.round(radiusKm)));
}

/** Keep a picked radius until a person's zoom or pan makes the viewport clearly larger or
 * smaller; camera moves the map makes itself never adopt it. Global doesn't grow. */
export function shouldAdoptViewportRadius(
  selectedKm: number,
  viewportKm: number,
  userInitiated: boolean,
): boolean {
  if (!userInitiated || !Number.isFinite(selectedKm) || !Number.isFinite(viewportKm)) return false;
  const selectedIsGlobal = selectedKm >= GLOBAL_RADIUS_KM * 0.6;
  const slack = Math.max(2, selectedIsGlobal ? 50 : selectedKm * 0.05);
  if (viewportKm > selectedKm + slack) return !selectedIsGlobal;
  return viewportKm < selectedKm - slack;
}

/** web's `normalizeRadiusFromUrl`: a link's radius ("global", or whole km). */
export function normalizeRadiusFromUrl(value: string | null | undefined): string | null {
  const trimmed = value?.trim();
  if (!trimmed) return null;
  if (trimmed === GLOBAL_RADIUS_VALUE) return GLOBAL_RADIUS_VALUE;
  const parsed = Number(trimmed);
  if (!Number.isFinite(parsed)) return trimmed;
  return parsed >= GLOBAL_RADIUS_KM ? GLOBAL_RADIUS_VALUE : String(Math.round(parsed));
}
