import { chosenZone, setDisplayZone } from '#lib/domain/zone.ts';

/**
 * web's Display timezone (Settings > Appearance), kept on this device like the app's other
 * appearance choices. Null follows the device's own zone.
 */
const KEY = 'sp_display_timezone';

function read(): string | null {
  try {
    return localStorage.getItem(KEY);
  } catch {
    return null;
  }
}

setDisplayZone(read());

export const displayZoneChoice = $state({ zone: chosenZone() });

export function chooseDisplayZone(zone: string | null): void {
  setDisplayZone(zone);
  displayZoneChoice.zone = chosenZone();
  try {
    if (displayZoneChoice.zone) localStorage.setItem(KEY, displayZoneChoice.zone);
    else localStorage.removeItem(KEY);
  } catch {
    // Private windows: the choice holds until the page closes.
  }
}
