// M1 accounts end to end, in Chromium, against a local seed node. Two kinds of device:
// one with no passkey store (like many Linux desktops) and one with a virtual passkey.
// Run `bash scripts/dev-seed.sh` (RP_ORIGIN must match the UI origin), serve the UI built
// with VITE_SP_GATEWAYS=http://localhost:8080, then:
//   node tests/e2e/accounts.mjs http://localhost:4174
import assert from 'node:assert/strict';

import { chromium } from 'playwright';

const BASE = process.argv[2] ?? 'http://localhost:5173';
const browser = await chromium.launch();
const log = (...a) => console.log(...a);
const errors = [];

/** A fresh browser profile; `passkeys` adds a device passkey store (virtual authenticator). */
async function device({ passkeys }) {
  const context = await browser.newContext();
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

const signedIn = (page) => page.getByRole('button', { name: 'Sign out' }).waitFor({ timeout: 120_000 });

async function signUp(page, name) {
  await page.getByRole('button', { name: 'Create an account' }).click();
  await page.getByLabel('Your name').fill(name);
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

async function phraseSignIn(page, words) {
  await page.getByRole('button', { name: 'Sign in with recovery phrase' }).click();
  await page.getByLabel('Recovery phrase').fill(words.join(' '));
  await page.getByRole('button', { name: 'Sign in', exact: true }).click();
  await signedIn(page);
}

async function keyOf(page) {
  await page.getByRole('link', { name: 'People' }).click();
  return (await page.locator('.key').innerText()).trim();
}

// 1. A device with no passkey store: everything works with the phrase, no passkey prompts.
const a = await device({ passkeys: false });
assert.equal(await a.getByRole('button', { name: 'Sign in with a passkey' }).count(), 0);
const t0 = Date.now();
const words = await signUp(a, 'Ann Tester');
log(`signed up in ${((Date.now() - t0) / 1000).toFixed(1)}s`);
await a.locator('.key').waitFor({ timeout: 60_000 });
assert.equal(await a.getByText('Save a passkey').count(), 0, 'no passkey offer without a store');
const annKey = await keyOf(a);

await a.getByRole('link', { name: 'Profile' }).click();
await a.getByLabel('Home area').fill('Brisbane, QLD');
await a.getByRole('button', { name: 'Save profile' }).click();
await a.getByText('Saved.').waitFor({ timeout: 60_000 });
await a.reload();
await signedIn(a);
log('profile saved; reload kept the session');

await a.getByRole('button', { name: 'Sign out' }).click();
await a.getByRole('button', { name: 'Sign in with recovery phrase' }).click();
await a.getByLabel('Recovery phrase').fill([...words.slice(0, 6), 'appel', ...words.slice(7)].join(' '));
await a.getByText(/Word 7 .appel. isn't in the word list/).waitFor({ timeout: 5000 });
await a.getByRole('button', { name: 'Back' }).click();
log('typo in the phrase is named');
await phraseSignIn(a, words);
assert.equal(await keyOf(a), annKey);
log('signed back in with the phrase (no passkey)');

// 2. A device with a passkey store: phrase sign-in, save a passkey, then use it.
const b = await device({ passkeys: true });
await phraseSignIn(b, words);
assert.equal(await keyOf(b), annKey, 'same identity on a second device');
await b.getByRole('button', { name: 'Save a passkey' }).click();
await b.getByText('Sign in faster on this device').waitFor({ state: 'detached', timeout: 60_000 });
await b.getByRole('button', { name: 'Sign out' }).click();
await b.getByRole('button', { name: 'Sign in with a passkey' }).click();
await signedIn(b);
assert.equal(await keyOf(b), annKey);
log('saved a passkey and signed in with it');

// 3. Pairing: another person's device asks to pair with Ann; Ann confirms.
const c = await device({ passkeys: false });
await signUp(c, 'Ann Laptop');
await c.getByRole('link', { name: 'Profile' }).click();
await c.getByPlaceholder('uhCAk…').fill(annKey);
await c.getByRole('button', { name: 'Send pairing request' }).click();
await c.getByText('Request sent').waitFor({ timeout: 60_000 });
await a.getByRole('link', { name: 'Profile' }).click();
for (let i = 0; i < 20 && !(await a.getByRole('button', { name: 'Confirm' }).count()); i++) {
  await a.reload();
  await signedIn(a);
  await a.getByRole('link', { name: 'Profile' }).click();
  await a.waitForTimeout(3000);
}
await a.getByRole('button', { name: 'Confirm' }).click();
await a.getByText(/^Paired:/).first().waitFor({ timeout: 60_000 });
await a.getByRole('link', { name: 'People' }).click();
await a.getByText('2 devices').first().waitFor({ timeout: 60_000 });
assert.equal(await a.locator('section.panel .chip', { hasText: 'Ann Laptop' }).count(), 0, 'paired device folded into its person');
log('devices paired and shown as one person');

assert.deepEqual(errors, [], 'no page errors');
log('all M1 account checks passed');
await browser.close();
