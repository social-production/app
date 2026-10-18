// M1 accounts end to end: Chromium with a virtual passkey authenticator against a local
// seed node. Run `bash scripts/dev-seed.sh` (RP_ORIGIN must match the UI origin), serve the
// UI built with VITE_SP_GATEWAYS=http://localhost:8080, then:
//   node tests/e2e/accounts.mjs http://localhost:4174
import assert from 'node:assert/strict';

import { chromium } from 'playwright';

const BASE = process.argv[2] ?? 'http://localhost:5173';
const browser = await chromium.launch();
const log = (...a) => console.log(...a);

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

async function device() {
  const context = await browser.newContext();
  const page = await context.newPage();
  const cdp = await context.newCDPSession(page);
  await cdp.send('WebAuthn.enable');
  await cdp.send('WebAuthn.addVirtualAuthenticator', {
    options: {
      protocol: 'ctap2', transport: 'internal', hasResidentKey: true,
      hasUserVerification: true, isUserVerified: true, automaticPresenceSimulation: true,
    },
  });
  const errors = [];
  page.on('pageerror', (e) => errors.push(String(e)));
  return { page, errors };
}

const signedIn = (page) => page.getByRole('button', { name: 'Sign out' }).waitFor({ timeout: 120_000 });

// 1. Sign up
const a = await device();
await a.page.goto(BASE);
const t0 = Date.now();
const words = await signUp(a.page, 'Ann Tester');
assert.equal(words.length, 24);
log(`signed up in ${((Date.now() - t0) / 1000).toFixed(1)}s`);

// 2. Profile
await a.page.getByRole('link', { name: 'Profile' }).click();
await a.page.getByLabel('Home area').fill('Brisbane, QLD');
await a.page.getByLabel('About you').fill('Testing M1.');
await a.page.getByRole('button', { name: 'Save profile' }).click();
await a.page.getByText('Saved.').waitFor({ timeout: 60_000 });
await a.page.getByRole('link', { name: 'People' }).click();
await a.page.getByText('Brisbane, QLD').first().waitFor({ timeout: 60_000 });
const myKey = (await a.page.locator('.key').innerText()).trim();
log('profile listed; key', myKey.slice(0, 12), '| standing:', await a.page.locator('.chip').first().innerText());

// 3. Reload resumes without a passkey prompt
await a.page.reload();
await signedIn(a.page);
log('reload resumed session');

// 4. Sign out, sign in with the passkey
await a.page.getByRole('button', { name: 'Sign out' }).click();
await a.page.getByRole('button', { name: 'Sign in' }).click();
await signedIn(a.page);
log('signed in with passkey');

// 5. Second device restores from the phrase: same agent
const b = await device();
await b.page.goto(BASE);
await b.page.getByRole('button', { name: 'Restore with recovery phrase' }).click();
await b.page.getByLabel('Recovery phrase').fill(words.join(' '));
await b.page.getByRole('button', { name: 'Restore' }).click();
await signedIn(b.page);
await b.page.locator('.key').waitFor({ timeout: 60_000 });
const restoredKey = (await b.page.locator('.key').innerText()).trim();
assert.equal(restoredKey, myKey, 'restore gives the same agent');
log('restored on device 2 as the same agent');

// 6. Pairing: a second person's device asks to pair with Ann; Ann confirms
const c = await device();
await c.page.goto(BASE);
await signUp(c.page, 'Ann Laptop');
await c.page.getByRole('link', { name: 'Profile' }).click();
await c.page.getByPlaceholder('uhCAk…').fill(myKey);
await c.page.getByRole('button', { name: 'Send pairing request' }).click();
await c.page.getByText('Request sent').waitFor({ timeout: 60_000 });
await a.page.getByRole('link', { name: 'Profile' }).click();
for (let i = 0; i < 20 && !(await a.page.getByRole('button', { name: 'Confirm' }).count()); i++) {
  await a.page.reload();
  await signedIn(a.page);
  await a.page.getByRole('link', { name: 'Profile' }).click();
  await a.page.waitForTimeout(3000);
}
await a.page.getByRole('button', { name: 'Confirm' }).click();
await a.page.getByText('Device paired.').waitFor({ timeout: 60_000 });
await a.page.getByText(/^Paired:/).first().waitFor({ timeout: 60_000 });
log('devices paired');

const errors = [...a.errors, ...b.errors, ...c.errors];
assert.deepEqual(errors, [], 'no page errors');
log('all M1 account checks passed');
await browser.close();
