// Shared browser-test helpers: devices (fresh browser profiles), sign-up and sign-in,
// against a local seed node (see accounts.mjs for how to run).
import { createHash } from 'node:crypto';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';

import { chromium } from 'playwright';

export const BASE = process.argv[2] ?? 'http://localhost:4174';
// CHROMIUM_PATH: a preinstalled Chromium (cloud sessions) instead of Playwright's download.
// A fake microphone, allowed without a prompt, for the call tests (calls.mjs).
export const browser = await chromium.launch({
  args: ['--use-fake-ui-for-media-stream', '--use-fake-device-for-media-stream'],
  ...(process.env.CHROMIUM_PATH ? { executablePath: process.env.CHROMIUM_PATH } : {}),
});
export const log = (...a) => console.log(...a);
/** Uncaught page errors from every device; tests assert it stays empty. */
export const errors = [];

/** A fresh browser profile; `passkeys` adds a device passkey store (virtual authenticator). */
export async function device({ passkeys = false, viewport } = {}) {
  const context = await browser.newContext(viewport ? { viewport } : {});
  const page = await context.newPage();
  const cdp = await context.newCDPSession(page);
  await cdp.send('WebAuthn.enable');
  if (passkeys) {
    await cdp.send('WebAuthn.addVirtualAuthenticator', {
      options: {
        protocol: 'ctap2', transport: 'internal', hasResidentKey: true,
        hasUserVerification: true, isUserVerified: true, automaticPresenceSimulation: true,
      },
    });
  }
  page.on('pageerror', (e) => errors.push(String(e)));
  if (process.env.UX_SHOTS) watch(page);
  await page.goto(BASE);
  return page;
}

// UX_SHOTS=<dir>: a screenshot of every device every four seconds, for reviewing a flow's
// screens afterwards (identical frames are skipped).
let devices = 0;
function watch(page) {
  const id = ++devices;
  let n = 0;
  let last = '';
  const timer = setInterval(async () => {
    const shot = await page.screenshot().catch(() => null);
    if (!shot) return page.isClosed() && clearInterval(timer);
    const key = createHash('sha1').update(shot).digest('hex');
    if (key === last) return;
    last = key;
    mkdirSync(process.env.UX_SHOTS, { recursive: true });
    writeFileSync(`${process.env.UX_SHOTS}/d${id}-${String(++n).padStart(3, '0')}.png`, shot);
  }, 4000);
  timer.unref();
}

// The shell marks itself once the account is connected (any screen width).
export const signedIn = (page) => page.locator('.shell[data-signed-in="true"]').waitFor({ timeout: 120_000 });
export const open = (page, name) => page.getByRole('link', { name, exact: true }).first().click();

/** Signed out, the home page is the public feed; sign-up and log-in live at /onboarding. */
export async function onboarding(page) {
  await page.goto(`${BASE}/onboarding`);
}

export async function signUp(page, name) {
  await onboarding(page);
  await page.getByRole('tab', { name: 'Sign up' }).click();
  await page.getByLabel('Username').fill(name);
  await page.getByRole('button', { name: 'Next' }).click();
  const words = (await page.locator('ol.grid li').allInnerTexts()).map((t) => t.split(/\s+/).pop());
  await page.getByRole('button', { name: "I've written them down" }).click();
  for (const label of await page.locator('label').allInnerTexts()) {
    const n = Number(label.match(/Word #(\d+)/)?.[1]);
    if (n) await page.getByLabel(`Word #${n}`, { exact: true }).fill(words[n - 1]);
  }
  await page.getByRole('button', { name: 'Create account' }).click();
  await signedIn(page);
  // The app then leaves onboarding for the feed (which has the full shell again).
  await page.waitForURL((url) => !url.pathname.startsWith('/onboarding'));
  return words;
}

export async function phraseSignIn(page, words) {
  await onboarding(page);
  await page.getByRole('tab', { name: 'Log in' }).click();
  await page.getByLabel('Recovery phrase').fill(words.join(' '));
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await signedIn(page);
  await page.waitForURL((url) => !url.pathname.startsWith('/onboarding'));
}

export async function signOut(page) {
  await open(page, 'Settings');
  await page.getByRole('button', { name: 'Sign out' }).click();
  // Signing out reloads the page; wait until it has, before navigating anywhere.
  await page.locator('.shell[data-signed-in="false"]').waitFor({ timeout: 60_000 });
  await page.waitForLoadState('load');
}

export async function keyOf(page) {
  await open(page, 'Settings');
  const key = page.locator('#settings-account .key');
  await key.waitFor({ timeout: 60_000 });
  return (await key.innerText()).trim();
}


/** web's right rail (the top bar's activity button). */
export const rail = (page) => page.locator('.right-rail .rail-panel');
export const railButton = (page) => page.getByRole('button', { name: 'Open activity' });

/** Open the rail (it reloads on open) until `locator` shows in it; `wait` per opening (a
 * founder in many projects waits longer for the rail to load). */
export async function railShows(page, locator, what, wait = 10_000) {
  for (let i = 0; i < 20; i++) {
    if (!(await rail(page).count())) await railButton(page).click();
    if (await locator().waitFor({ timeout: wait }).then(() => true, () => false)) return;
    await railButton(page).click(); // close, then reopen to reload it
  }
  throw new Error(`timed out waiting for ${what} in the rail`);
}

/** The channel browser tests tag their projects, events and help requests with (web requires
 * one, ADR 0068). Founder one makes it once per run; a taken handle means it already exists. */
export const TEST_CHANNEL = 'testing';
let testChannel;
export function ensureTestChannel() {
  testChannel ??= (async () => {
    const [one] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
    const page = await device();
    await phraseSignIn(page, one.phrase.split(' '));
    await page.goto(`${BASE}/create/channel`);
    await page.getByLabel(/Channel handle/).fill(TEST_CHANNEL);
    await page.getByLabel(/Description/).fill('Made by the browser tests.');
    // On a fresh node the week's first write can land beside this one ("the source chain
    // head has moved"): press again.
    const moved = page.getByText('head has moved');
    for (let i = 0; i < 4; i++) {
      await page.getByRole('button', { name: 'Create channel' }).click();
      await page
        .getByRole('heading', { name: TEST_CHANNEL })
        .or(page.getByText('already used'))
        .or(moved)
        .first()
        .waitFor({ timeout: 120_000 });
      if (!(await moved.isVisible())) break;
      await page.waitForTimeout(2000);
    }
    await page.context().close();
  })();
  return testChannel;
}

/** On a create flow's scope step: types `name` into the tag field labelled `label` and picks it.
 * The index lists a new channel within a few minutes, so keep searching until it does. */
export async function pickScopeTag(page, label, name) {
  const field = page.getByLabel(label, { exact: true });
  const option = page.getByRole('option', { name, exact: true });
  for (let i = 0; ; i++) {
    await field.fill(name);
    // The search answers asynchronously: wait for it, not just count what is there now.
    if (await option.waitFor({ timeout: 5000 }).then(() => true, () => false)) break;
    if (i >= 60) throw new Error(`${name} never appeared in the "${label}" search`);
    await field.fill('');
    await page.waitForTimeout(2000);
  }
  await option.click();
}

/** On a create flow's scope step: picks the test channel as the primary tag (or, with `label`,
 * in another tag field). */
export async function tagTestChannel(page, label = 'Primary channel tag') {
  await ensureTestChannel();
  await pickScopeTag(page, label, TEST_CHANNEL);
}

/** Clicks Continue `n` times, tagging the test channel first whenever it is on a scope step
 * (a project's or help request's primary channel, or an event's channel tags). */
export async function continueSteps(page, n = 1) {
  for (let i = 0; i < n; i++) {
    for (const label of ['Primary channel tag', 'Channel tags']) {
      if (await page.getByLabel(label, { exact: true }).count()) {
        await tagTestChannel(page, label);
        break;
      }
    }
    await page.getByRole('button', { name: 'Continue' }).click();
  }
}

/** The feed's scope tabs (ADR 0102): 'Home', 'Global' or 'Region'. */
export const feedScope = (page, name) =>
  page.getByRole('tablist', { name: 'Choose public feed scope' }).getByRole('tab', { name, exact: true }).click();

/** One choice in the feed's filter sheet (ADR 0102): `group` is the menu's old name
 * ('Sort public feed by', 'Choose personal audience', 'Filter public feed', 'Public feed time
 * window'), `option` the choice's label. */
export async function feedChoice(page, group, option) {
  await page.getByRole('button', { name: 'Feed filters' }).click();
  const sheet = page.getByRole('dialog', { name: /^Show in / });
  await sheet.getByRole('radiogroup', { name: group }).getByRole('radio', { name: option, exact: true }).click();
  await sheet.getByRole('button', { name: 'Show results' }).click();
  await sheet.waitFor({ state: 'hidden' });
}
