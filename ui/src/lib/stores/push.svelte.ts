import {
  inNativeApp,
  pushIsOn,
  pushSupported,
  turnPushOff,
  turnPushOn,
  updatePushChoices,
} from '#lib/holo/index.ts';

import { notificationsOff } from './notifications.svelte.ts';

/**
 * Push notifications on this device (ADR 0108). `where`: 'browser' can get pushes, 'desktop'
 * (the app shows its own, later), 'none' (this browser can't; iPhones only from the Home Screen).
 */
export const push = $state<{
  where: 'browser' | 'desktop' | 'none';
  on: boolean;
  details: boolean;
  busy: boolean;
  problem: string;
}>({ where: 'none', on: false, details: readDetails(), busy: false, problem: '' });

const DETAILS_KEY = 'sp_push_details';

function readDetails(): boolean {
  try {
    return localStorage.getItem(DETAILS_KEY) !== 'off';
  } catch {
    return true;
  }
}

function choices() {
  return { muted: [...notificationsOff.categories], details: push.details };
}

/** Read this device's state (Settings opens). */
export async function loadPush(): Promise<void> {
  push.where = inNativeApp() ? 'desktop' : pushSupported() ? 'browser' : 'none';
  push.on = push.where === 'browser' && (await pushIsOn().catch(() => false));
}

async function busy(work: () => Promise<void>): Promise<void> {
  push.busy = true;
  push.problem = '';
  try {
    await work();
  } catch (e) {
    push.problem = e instanceof Error ? e.message : String(e);
  } finally {
    push.busy = false;
  }
}

export function setPushOn(on: boolean): Promise<void> {
  return busy(async () => {
    if (!on) {
      await turnPushOff();
      push.on = false;
      return;
    }
    push.on = await turnPushOn(choices());
    if (!push.on)
      push.problem = 'Notifications are blocked for this site in your browser settings.';
  });
}

export function setPushDetails(details: boolean): Promise<void> {
  push.details = details;
  try {
    localStorage.setItem(DETAILS_KEY, details ? 'on' : 'off');
  } catch {
    // Private windows: the choice holds until the page closes.
  }
  return pushChoicesChanged();
}

/** The categories or "Show what it's about" changed: tell the gateway. */
export function pushChoicesChanged(): Promise<void> {
  if (!push.on) return Promise.resolve();
  return busy(() => updatePushChoices(choices()));
}
