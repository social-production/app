import { t } from '#lib/i18n/index.ts';
import {
  inNativeApp,
  osNotificationsAllowed,
  osNotify,
  pushIsOn,
  pushSupported,
  turnPushOff,
  turnPushOn,
  updatePushChoices,
  type Notice,
} from '#lib/holo/index.ts';

import { notificationsOff } from './notifications.svelte.ts';

/**
 * Push notifications on this device (ADR 0108). `where`: 'browser' can get pushes, 'desktop'
 * (the app shows the OS's own while it runs), 'none' (this browser can't; iPhones only from the
 * Home Screen).
 */
export const push = $state<{
  where: 'browser' | 'desktop' | 'none';
  on: boolean;
  details: boolean;
  busy: boolean;
  problem: string;
}>({ where: 'none', on: false, details: readDetails(), busy: false, problem: '' });

const DETAILS_KEY = 'sp_push_details';
/** The desktop app's switch: on unless switched off (the OS may still refuse). */
const DESKTOP_KEY = 'sp_desktop_notifications';

function readDesktop(): boolean {
  try {
    return localStorage.getItem(DESKTOP_KEY) !== 'off';
  } catch {
    return true;
  }
}

function saveDesktop(on: boolean): void {
  try {
    localStorage.setItem(DESKTOP_KEY, on ? 'on' : 'off');
  } catch {
    // Storage refused: the choice holds until the app closes.
  }
}

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
  if (push.where === 'desktop') {
    push.on = readDesktop() && (await osNotificationsAllowed().catch(() => false));
    return;
  }
  push.on = push.where === 'browser' && (await pushIsOn().catch(() => false));
}

/**
 * A notice reached the desktop app (ADR 0108): the OS shows it when the window isn't in front,
 * its category is on and the switch is on. In front, the bell already shows it.
 */
export function showOnDesktop(notice: Notice): void {
  if (!inNativeApp() || !readDesktop() || document.hasFocus()) return;
  if ((notificationsOff.categories as string[]).includes(notice.category)) return;
  const title = push.details ? notice.title : t('Social Production');
  const body = push.details ? notice.body : t('Something new is waiting for you.');
  void osNotify(title, body).catch(() => undefined);
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
    if (push.where === 'desktop') {
      push.on = on && (await osNotificationsAllowed());
      saveDesktop(on);
      if (on && !push.on)
        push.problem = t('Notifications are off for the app in your system settings.');
      return;
    }
    if (!on) {
      await turnPushOff();
      push.on = false;
      return;
    }
    push.on = await turnPushOn(choices());
    if (!push.on)
      push.problem = t('Notifications are blocked for this site in your browser settings.');
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
  // The desktop app reads the categories as each notice arrives.
  if (!push.on || push.where === 'desktop') return Promise.resolve();
  return busy(() => updatePushChoices(choices()));
}
