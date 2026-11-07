/**
 * Places (M6, ADR 0078/0080): web's `LocationPickerValue` (`types/locationPicker.ts`), the
 * gateway's search results (web's `Location`, ADR 0079) and the zome's `Place` record.
 */

export type PlacePrecision = 'exact' | 'approximate';
export type PlaceMode = 'physical' | 'online' | 'tbd';

/** web's LocationPickerValue, without `locationId` (the app keeps the place on the record). */
export interface PlaceValue {
  mode: PlaceMode;
  displayLabel: string;
  providerPlaceId: string | null;
  latitude: number | null;
  longitude: number | null;
  region: string | null;
  country: string | null;
  precision: PlacePrecision;
  isOnline: boolean;
}

/** web's LocationQuickPick. */
export interface PlaceQuickPick {
  id: string;
  label: string;
  sourceLabel: string;
  value: PlaceValue;
}

/** The zome's `project_integrity::Place`. */
export interface Place {
  label: string;
  latitude: number | null;
  longitude: number | null;
  region: string | null;
  country: string | null;
  provider_id: string | null;
  precision: PlacePrecision;
  online: boolean;
}

/** A result from the gateway's `/locations/*` routes (web's backend shape). */
export interface GatewayLocation {
  provider_place_id: string | null;
  display_label: string;
  latitude: number | null;
  longitude: number | null;
  region: string | null;
  country: string | null;
  precision: string;
  is_online: boolean;
}

export function emptyPlaceValue(mode: PlaceMode = 'physical'): PlaceValue {
  return {
    mode,
    displayLabel: '',
    providerPlaceId: null,
    latitude: null,
    longitude: null,
    region: null,
    country: null,
    precision: 'approximate',
    isOnline: mode === 'online',
  };
}

export function onlinePlaceValue(label = 'Online'): PlaceValue {
  return { ...emptyPlaceValue('online'), displayLabel: label };
}

/** A physical place named by hand: no coordinates (web's commitCustomPhysical). */
export function typedPlaceValue(label: string): PlaceValue {
  return { ...emptyPlaceValue('physical'), displayLabel: label };
}

/** web's `mapLocation`. */
export function fromGateway(item: GatewayLocation): PlaceValue {
  return {
    mode: item.is_online ? 'online' : 'physical',
    displayLabel: item.display_label,
    providerPlaceId: item.provider_place_id,
    latitude: item.latitude,
    longitude: item.longitude,
    region: item.region,
    country: item.country,
    precision: item.precision === 'exact' ? 'exact' : 'approximate',
    isOnline: item.is_online,
  };
}

/** A zome place back into the picker's value (an edit sheet or wizard opening on it). */
export function fromPlace(place: Place | null | undefined, label: string): PlaceValue {
  if (!place) return label.trim() ? typedPlaceValue(label) : emptyPlaceValue();
  if (place.online) return onlinePlaceValue(place.label);
  return {
    mode: 'physical',
    displayLabel: place.label,
    providerPlaceId: place.provider_id,
    latitude: place.latitude,
    longitude: place.longitude,
    region: place.region,
    country: place.country,
    precision: place.precision,
    isOnline: false,
  };
}

/** web-backend's coordinate rounding (2 decimals approximate, 6 exact; `sp_place::round`). */
export function roundCoordinate(value: number, precision: PlacePrecision): number {
  const scale = 10 ** (precision === 'exact' ? 6 : 2);
  return Math.round(value * scale) / scale;
}

function area(value: string | null, max: number): string | null {
  const trimmed = value?.trim() ?? '';
  return trimmed ? [...trimmed].slice(0, max).join('') : null;
}

/** Cut a label to `max` characters (a record's label limit can be below the place's 240). */
export function fitLabel(label: string, max: number): string {
  return [...label.trim()].slice(0, max).join('').trim();
}

/**
 * The record's place for a picked value, as web-backend's `create_location` saves it: an
 * online place, or a physical one with coordinates rounded to its precision. A typed label
 * without coordinates has no place (as on web). `label` is the record's own (cut) label.
 */
export function placeRecord(value: PlaceValue, label: string): Place | null {
  if (!label.trim()) return null;
  if (value.mode === 'online' || value.isOnline) {
    return {
      label,
      latitude: null,
      longitude: null,
      region: null,
      country: null,
      provider_id: null,
      precision: 'approximate',
      online: true,
    };
  }
  if (value.mode !== 'physical' || value.latitude == null || value.longitude == null) return null;
  return {
    label,
    latitude: roundCoordinate(value.latitude, value.precision),
    longitude: roundCoordinate(value.longitude, value.precision),
    region: area(value.region, 120),
    country: area(value.country, 120),
    provider_id: area(value.providerPlaceId, 160),
    precision: value.precision,
    online: false,
  };
}

/** The place a create flow's Location step gives `label` (web's location intent). */
export function intentPlace(
  intent: 'physical' | 'online' | 'later',
  value: PlaceValue,
  label: string,
): Place | null {
  if (intent === 'online') return placeRecord(onlinePlaceValue(label), label);
  return intent === 'physical' ? placeRecord(value, label) : null;
}

/** web's `buildLocationQuickPick`: null for a blank or "Online" label. */
export function quickPick(input: {
  id: string;
  label: string;
  sourceLabel: string;
  place?: Place | null;
}): PlaceQuickPick | null {
  const label = input.label.trim();
  if (!label || label.toLowerCase() === 'online') return null;
  const value =
    input.place && !input.place.online ? fromPlace(input.place, label) : typedPlaceValue(label);
  return {
    id: input.id,
    label,
    sourceLabel: input.sourceLabel,
    value: { ...value, displayLabel: label },
  };
}

/** web's `dedupeLocationQuickPicks` (by label; the app has no location ids). */
export function dedupeQuickPicks(picks: (PlaceQuickPick | null | undefined)[]): PlaceQuickPick[] {
  const seen = new Set<string>();
  return picks.filter((pick): pick is PlaceQuickPick => {
    if (!pick || seen.has(pick.label.toLowerCase())) return false;
    seen.add(pick.label.toLowerCase());
    return true;
  });
}

/** web's isQuickPickSelected, by label. */
export function isPickSelected(pick: PlaceQuickPick, value: PlaceValue): boolean {
  if (value.mode !== 'physical' || value.isOnline) return false;
  return pick.label.trim().toLowerCase() === value.displayLabel.trim().toLowerCase();
}
