// Shared browser-test helpers: devices (fresh browser profiles), sign-up and sign-in,
// against a local seed node (see accounts.mjs for how to run).
import { readFileSync } from 'node:fs';

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
  await page.goto(BASE);
  return page;
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
    await page.getByRole('button', { name: 'Create channel' }).click();
    await page
      .getByRole('heading', { name: TEST_CHANNEL })
      .or(page.getByText('already used'))
      .first()
      .waitFor({ timeout: 120_000 });
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

/** Opens the plan wizard from "Add plan". A proposal has no type yet (ADR 0059), so it asks
 * which kind of plan first ('Production plan', 'Operations plan' or 'Event plan'); a project
 * that already has its type goes straight to the wizard. */
export async function addPlan(page, kind = 'Production plan') {
  await page.getByRole('button', { name: 'Add plan' }).click();
  const choice = page.locator('.plan-kind-choice');
  await choice.or(page.getByRole('dialog')).first().waitFor({ timeout: 30_000 });
  if (!(await choice.isVisible())) return;
  await choice.getByRole('radio', { name: new RegExp(kind) }).click();
  await choice.getByRole('button', { name: 'Continue' }).click();
}
