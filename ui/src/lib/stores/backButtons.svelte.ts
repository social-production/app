/**
 * Owner's fix 3f (2026-11-17): "Show back buttons", on by default and kept on this device.
 * Android has its own back gesture, so someone can hide the round back buttons (the detail
 * dock, sign-up, the comment bar); the dock's capsule then stays in the middle on its own.
 */
const KEY = 'sp_back_buttons';

function read(): boolean {
  try {
    return localStorage.getItem(KEY) !== 'off';
  } catch {
    return true;
  }
}

export const backButtons = $state({ shown: read() });

export function setBackButtons(shown: boolean): void {
  backButtons.shown = shown;
  try {
    localStorage.setItem(KEY, shown ? 'on' : 'off');
  } catch {
    // Private windows: the choice holds until the page closes.
  }
}

/** Puts the choice on <html> (`data-back-buttons`), where backbuttons.css reads it. */
export function applyBackButtons(): void {
  document.documentElement.dataset.backButtons = backButtons.shown ? 'on' : 'off';
}
