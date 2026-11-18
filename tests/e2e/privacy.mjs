// Privacy switches (ADR 0112), display rules as on web: Ann turns on "Hide profile activity
// from non-followers" and "Hide my public activity from others' personal feeds". A guest then
// sees a note instead of her profile's feed, while Bob (a follower) still sees her proposal
// there; it leaves Bob's Home feed.
//   node tests/e2e/privacy.mjs http://localhost:4174 [screens dir]
// It signs up two test accounts, so it may run against the live test network too.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, continueSteps, device, errors, feedScope, log, signUp } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const title = `Seed swap ${stamp}`;

/** Reload `page` at `url` until `check` passes (network reads lag). */
async function until(page, url, check, what) {
  const t0 = Date.now();
  for (;;) {
    await page.goto(url);
    for (let i = 0; i < 10; i++) {
      if (await check().catch(() => false)) return;
      await page.waitForTimeout(2000);
    }
    if (Date.now() - t0 > 180_000) throw new Error(`timed out waiting for ${what}`);
  }
}

/** The agent key from Settings. */
async function keyOf(page) {
  await page.goto(`${BASE}/settings`);
  const key = page.locator('#settings-account .key');
  await key.waitFor({ timeout: 60_000 });
  return (await key.innerText()).trim();
}

const ann = await device({ viewport: phone });
await signUp(ann, `ann${stamp}`);
const annProfile = `${BASE}/profile/${await keyOf(ann)}`;
await ann.goto(`${BASE}/create/project`);
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('What do you propose?').fill('Swap seeds in spring.');
await continueSteps(ann, 3);
await ann.getByRole('button', { name: 'Publish proposal' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
log('ann proposed something');

const bob = await device({ viewport: phone });
await signUp(bob, `bob${stamp}`);
await until(bob, annProfile, () => bob.getByRole('button', { name: `Follow ann${stamp}` }).isVisible(), 'ann');
await bob.getByRole('button', { name: `Follow ann${stamp}` }).click();
await bob.getByRole('button', { name: `Unfollow ann${stamp}` }).waitFor({ timeout: 60_000 });
await bob.goto(`${BASE}/`);
await feedScope(bob, 'Home');
await until(bob, `${BASE}/`, () => bob.locator('.project-card', { hasText: title }).isVisible(), 'ann\'s proposal on Home');
log('bob follows ann; her proposal is on his Home');

await ann.goto(`${BASE}/settings`);
for (const name of ['Hide profile activity from non-followers', 'Hide my public activity from others’ personal feeds']) {
  const toggle = ann.getByRole('switch', { name });
  await toggle.click();
  for (let i = 0; (await toggle.getAttribute('aria-checked')) !== 'true'; i++) {
    if (i > 60) throw new Error(`${name} never turned on`);
    await ann.waitForTimeout(1000);
  }
}
await ann.locator('#settings-privacy').scrollIntoViewIfNeeded();
await ann.screenshot({ path: `${out}/phone390-privacy-settings.png` });
await ann.emulateMedia({ colorScheme: 'dark' });
await ann.screenshot({ path: `${out}/phone390-privacy-settings-dark.png` });
log('ann turns on two privacy switches');

const guest = await device({ viewport: phone });
const note = () => guest.getByText(`ann${stamp} shares their activity with followers`);
await until(guest, annProfile, () => note().isVisible(), 'the followers-only note');
assert.equal(await guest.locator('.project-card', { hasText: title }).count(), 0);
await guest.screenshot({ path: `${out}/phone390-privacy-guest-profile.png` });
log('a guest sees the note, not her feed');

await until(bob, annProfile, () => bob.locator('.project-card', { hasText: title }).isVisible(), 'her feed for a follower');
assert.equal(await bob.getByText(`ann${stamp} shares their activity with followers`).count(), 0);
log('bob, a follower, still sees her profile feed');

await bob.goto(`${BASE}/`);
await feedScope(bob, 'Home');
await until(
  bob,
  `${BASE}/`,
  async () => !(await bob.locator('.feed-loading, .skeleton-row').count()) &&
    !(await bob.locator('.project-card', { hasText: title }).count()),
  'her proposal to leave Home',
);
log('her proposal left bob\'s Home');

assert.deepEqual(errors, [], 'no page errors');
await browser.close();
log('privacy ok');
