// Shared browser-test helpers: devices (fresh browser profiles), sign-up and sign-in,
// against a local seed node (see accounts.mjs for how to run).
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

