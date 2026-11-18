/**
 * web's Display timezone (Settings > Appearance): scheduled activity and event times show,
 * and are entered, in the chosen zone instead of the device's. The date code reads wall
 * clocks with a Date's local getters, so `wall()` shifts an instant so those getters give
 * the chosen zone's clock, and `fromWall()` turns such a shifted Date back into the instant.
 * With no zone chosen (or the device's own) both return the Date unchanged.
 */
let chosen: string | null = null;

export const deviceZone = (): string => Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC';

export function validZone(zone: string): boolean {
  try {
    new Intl.DateTimeFormat('en-US', { timeZone: zone });
    return true;
  } catch {
    return false;
  }
}

export function setDisplayZone(zone: string | null): void {
  chosen = zone && validZone(zone) && zone !== deviceZone() ? zone : null;
}

/** The zone times show in: the chosen one, else the device's. */
export const displayZone = (): string => chosen ?? deviceZone();

/** The chosen zone, or null when times follow the device. */
export const chosenZone = (): string | null => chosen;

/** Every zone the browser knows, sorted, for the picker. */
export function allZones(): string[] {
  const intl = Intl as typeof Intl & { supportedValuesOf?: (key: string) => string[] };
  const zones = intl.supportedValuesOf?.('timeZone') ?? [deviceZone(), 'UTC'];
  return [...new Set(zones)].sort();
}

/** `zone`'s offset from UTC at `ms`, in ms. */
export function zoneOffset(zone: string, ms: number): number {
  const parts = new Intl.DateTimeFormat('en-US', {
    timeZone: zone,
    hourCycle: 'h23',
    year: 'numeric',
    month: 'numeric',
    day: 'numeric',
    hour: 'numeric',
    minute: 'numeric',
    second: 'numeric',
  }).formatToParts(new Date(ms));
  const n = (t: string): number => Number(parts.find((p) => p.type === t)?.value ?? 0);
  const asUtc = Date.UTC(n('year'), n('month') - 1, n('day'), n('hour'), n('minute'), n('second'));
  return asUtc - Math.floor(ms / 1000) * 1000;
}

const deviceOffset = (ms: number): number => -new Date(ms).getTimezoneOffset() * 60_000;

/** A Date whose local getters read the display zone's wall clock at `ms`. */
export function wall(ms: number): Date {
  if (!chosen) return new Date(ms);
  return new Date(ms + zoneOffset(chosen, ms) - deviceOffset(ms));
}

/** The instant a Date built from a typed day and time (`new Date('YYYY-MM-DDTHH:MM')`) means
 * in the display zone. */
export function fromWall(local: Date): Date {
  if (!chosen) return local;
  const ms = local.getTime() + deviceOffset(local.getTime());
  let at = ms - zoneOffset(chosen, ms);
  at = ms - zoneOffset(chosen, at);
  return new Date(at);
}

/** "AEDT" for the display zone at `ms`, or "" when the browser gives only GMT±n. */
export function zoneName(ms: number): string {
  const zone = displayZone();
  const locale = zone.startsWith('Australia/') ? 'en-AU' : undefined;
  const part = new Intl.DateTimeFormat(locale, { timeZone: zone, timeZoneName: 'short' })
    .formatToParts(new Date(ms))
    .find((p) => p.type === 'timeZoneName')?.value;
  return part && !/^GMT[+-]/i.test(part) ? part : '';
}

/** Format options in the display zone. */
export const inZone = (o: Intl.DateTimeFormatOptions): Intl.DateTimeFormatOptions => ({
  ...o,
  timeZone: displayZone(),
});

/** A typed `YYYY-MM-DDTHH:MM` value, read in the display zone, as Holochain micros (NaN if bad). */
export const wallMicros = (local: string): number => fromWall(new Date(local)).getTime() * 1000;
