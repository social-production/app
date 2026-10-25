// Follows and platform feedback (ADR 0054), as on web: Follow → Following, approval turns new
// follows into requests the person accepts in Settings; Feedback lists bugs and suggestions
// with votes, and guests can read it.
//   node tests/e2e/follows.mjs http://localhost:4174 [screens dir]
// It signs up three test accounts, so it may run against the live test network too.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, device, errors, log, signUp } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);

/** Reload `page` at `url` until `check` finds what it waits for (network reads lag). */
async function until(page, url, check, what) {
  const t0 = Date.now();
  for (;;) {
    await page.goto(url);
    // Give the page time to render and read before reloading.
    for (let i = 0; i < 10; i++) {
      if (await check().catch(() => false)) return;
      await page.waitForTimeout(2000);
    }
    if (Date.now() - t0 > 180_000) throw new Error(`timed out waiting for ${what}`);
  }
}

/** The agent key from Settings (on a phone the Settings link sits in the More sheet). */
async function keyOf(page) {
  await page.goto(`${BASE}/settings`);
  const key = page.locator('#settings-account .key');
  await key.waitFor({ timeout: 60_000 });
  return (await key.innerText()).trim();
}

const ann = await device({ viewport: phone });
await signUp(ann, `ann${stamp}`);
const annKey = await keyOf(ann);
const annProfile = `${BASE}/profile/${annKey}`;

const bob = await device({ viewport: phone });
await signUp(bob, `bob${stamp}`);
await until(bob, annProfile, () => bob.getByRole('heading', { name: `ann${stamp}` }).isVisible(), 'ann');
await bob.getByRole('button', { name: `Follow ann${stamp}` }).click();
await bob.getByRole('button', { name: `Unfollow ann${stamp}` }).waitFor({ timeout: 60_000 });
await bob.screenshot({ path: `${out}/phone390-following.png` });
log('bob follows ann at once (no approval)');

await until(
  ann,
  annProfile,
  async () => (await ann.locator('.count-row button').first().innerText()).startsWith('1'),
  'one follower',
);
await ann.locator('.count-row button').first().click();
await ann.getByRole('dialog', { name: 'Followers' }).getByText(`bob${stamp}`).waitFor();
await ann.keyboard.press('Escape');
log('ann sees bob in her followers');

// Approval on: Cat's follow becomes a request until Ann accepts it.
await ann.goto(`${BASE}/settings`);
const approval = ann.getByRole('switch', { name: 'Require approval to follow you' });
await approval.click();
await ann.waitForFunction(
  () => document.querySelector('#settings-privacy .switch')?.getAttribute('aria-checked') === 'true',
);
const cat = await device({ viewport: phone });
await signUp(cat, `cat${stamp}`);
await until(cat, annProfile, () => cat.getByRole('heading', { name: `ann${stamp}` }).isVisible(), 'ann');
await cat.getByRole('button', { name: `Follow ann${stamp}` }).click();
await cat.getByRole('button', { name: /Cancel follow request/ }).waitFor({ timeout: 60_000 });
assert.equal(await cat.locator('.action.primary .action-text').innerText(), 'Requested');
log('with approval on, cat\'s follow is a request');

await until(
  ann,
  `${BASE}/settings`,
  () => ann.locator('#settings-follow-requests').getByText(`@cat${stamp}`).isVisible(),
  'the request in Settings',
);
await ann.screenshot({ path: `${out}/phone390-follow-requests.png` });
await ann.locator('#settings-follow-requests').getByRole('button', { name: 'Accept' }).click();
await ann.locator('#settings-follow-requests').waitFor({ state: 'detached', timeout: 60_000 });
await until(
  cat,
  annProfile,
  () => cat.getByRole('button', { name: `Unfollow ann${stamp}` }).isVisible(),
  'the accepted follow',
);
log('ann accepts; cat is following');

// Feedback: Ann files a suggestion; Bob supports it; a guest reads it.
await ann.goto(`${BASE}/feedback`);
await ann.getByRole('button', { name: 'Add feedback' }).click();
const compose = ann.getByRole('dialog', { name: 'Add feedback' });
await compose.getByRole('radio', { name: 'Suggestion' }).click();
await compose.getByLabel('Title').fill(`Dark tiles ${stamp}`);
await compose.getByLabel('Description').fill('The map is bright at night.');
await compose.getByRole('button', { name: 'Submit feedback' }).click();
await compose.waitFor({ state: 'detached', timeout: 60_000 });
await ann.getByText(`Dark tiles ${stamp}`).waitFor();
log('ann adds feedback');

await until(bob, `${BASE}/feedback`, () => bob.getByText(`Dark tiles ${stamp}`).isVisible(), 'feedback');
const card = bob.locator('.feedback-card', { hasText: `Dark tiles ${stamp}` });
await card.getByRole('button', { name: /^Support/ }).click();
await card.locator('.signal-percent', { hasText: '100%' }).waitFor({ timeout: 60_000 });
await card.click();
const detail = bob.getByRole('dialog', { name: 'Suggestion' });
await detail.getByText('The map is bright at night.').waitFor();
await bob.screenshot({ path: `${out}/phone390-feedback-detail.png` });
await bob.keyboard.press('Escape');
log('bob supports it and opens the detail');

const guest = await device({ viewport: phone });
await until(guest, `${BASE}/feedback`, () => guest.getByText(`Dark tiles ${stamp}`).isVisible(), 'guest feedback');
await guest.screenshot({ path: `${out}/phone390-feedback-guest.png` });
await guest.getByRole('button', { name: 'Add feedback' }).click();
await guest.getByText('Sign in to share feedback.').waitFor();
log('guests read feedback; adding asks them to sign in');

assert.deepEqual(errors, [], 'no page errors');
log('all follows and feedback checks passed');
await browser.close();
