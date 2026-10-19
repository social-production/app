// Shared browser-test helpers: devices (fresh browser profiles), sign-up and sign-in,
// against a local seed node (see accounts.mjs for how to run).
import { chromium } from 'playwright';

export const BASE = process.argv[2] ?? 'http://localhost:4174';
export const browser = await chromium.launch();
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

export async function signUp(page, name) {
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
  return words;
}

export async function phraseSignIn(page, words) {
  await page.getByRole('tab', { name: 'Log in' }).click();
  await page.getByLabel('Recovery phrase').fill(words.join(' '));
  await page.getByRole('button', { name: 'Log in', exact: true }).click();
  await signedIn(page);
}

export async function signOut(page) {
  await open(page, 'Settings');
  await page.getByRole('button', { name: 'Sign out' }).click();
}

export async function keyOf(page) {
  await open(page, 'Settings');
  const key = page.locator('#settings-account .key');
  await key.waitFor({ timeout: 60_000 });
  return (await key.innerText()).trim();
}

