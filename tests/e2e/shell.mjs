// The phone shell against web's (390px): the bottom nav's More opens web's MobileMoreSheet
// (profile or "Sign in / Sign up", Settings when signed in, About), not the left rail.
//   node tests/e2e/shell.mjs http://localhost:4174 [screens dir]
// It signs up one test account, so it may run against the live test network too.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, device, errors, log, signUp } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const sheet = (page) => page.getByRole('dialog', { name: 'More menu' });
// If the More tab can't be clicked, say where the page was (seen twice, not reproduced).
const more = (page) => ({
  click: async () => {
    try {
      await page
        .locator('.mobile-bottom-nav')
        .getByRole('button', { name: 'More' })
        .click({ timeout: 30_000 });
    } catch (e) {
      // Nothing here may wait: the page may not have rendered at all.
      await page.screenshot({ path: `${out}/more-fail.png` }).catch(() => {});
      log(
        'More not clickable at',
        page.url(),
        'navs',
        await page.locator('.mobile-bottom-nav').count(),
      );
      log(await page.evaluate(() => document.body.innerText.slice(0, 400)).catch(String));
      throw e;
    }
  },
});

const guest = await device({ viewport: phone });
await more(guest).click();
await sheet(guest).getByRole('link', { name: 'Sign in / Sign up' }).waitFor();
assert.equal(await sheet(guest).getByRole('link', { name: 'Settings' }).count(), 0);
assert.equal(await guest.locator('.left-rail[data-open="true"]').count(), 0, 'the rail stays shut');
await guest.screenshot({ path: `${out}/phone390-more-guest.png` });
await sheet(guest).getByRole('link', { name: 'About' }).click();
await guest.waitForURL(/\/about$/);
await sheet(guest).waitFor({ state: 'detached' });
assert.equal(
  await guest.getByRole('complementary', { name: 'About Social Production' }).count(),
  0,
  'no intro on About',
);
// web's About page: the essay in tabs, the open tab in ?section=.
await guest.getByRole('heading', { name: /Collective coordination of activity/ }).waitFor();
await guest.getByRole('tab', { name: /Strategy/ }).click();
await guest.waitForURL(/section=strategy/);
await guest.getByRole('heading', { name: 'Strategy', exact: true }).waitFor();
log('signed out: More shows sign in and About');

// web's isAuthSurface: onboarding shows only the brand.
await guest.goto(`${BASE}/onboarding`);
await guest.getByRole('heading', { name: 'Sign in or create an account' }).waitFor();
assert.equal(await guest.locator('.mobile-bottom-nav').count(), 0, 'no bottom nav');
assert.equal(
  await guest.getByRole('button', { name: 'Open networks' }).count(),
  0,
  'no rail toggle',
);
await guest.screenshot({ path: `${out}/phone390-onboarding.png` });
log('onboarding shows only the brand, as on web');

const ann = await device({ viewport: phone });
await signUp(ann, `ann${stamp}`);
await more(ann).click();
await sheet(ann)
  .getByRole('link', { name: `ann${stamp}` })
  .waitFor({ timeout: 30_000 });
await ann.screenshot({ path: `${out}/phone390-more-signed-in.png` });
await ann.keyboard.press('Escape');
await sheet(ann).waitFor({ state: 'detached' });
await more(ann).click();
await sheet(ann).getByRole('link', { name: 'Settings' }).click();
await ann.waitForURL(/\/settings/);
await more(ann).click();
await sheet(ann)
  .getByRole('link', { name: `ann${stamp}` })
  .click();
await ann.waitForURL(/\/profile$/);
log('signed in: More shows the profile, Settings and About');

// web's Search page finds people (the gateway's people index refreshes every minute).
const row = guest.locator('.result-card', { hasText: `ann${stamp}` });
for (let i = 0; !(await row.count()); i++) {
  if (i >= 15) throw new Error(`ann${stamp} not found by search`);
  await guest.goto(`${BASE}/search?q=${`ann${stamp}`}`);
  await guest.getByRole('heading', { name: 'Search', exact: true }).waitFor();
  await guest.waitForTimeout(6000);
}
assert.equal(await row.locator('.kind-chip').innerText(), 'Profile');
await guest.screenshot({ path: `${out}/phone390-search.png` });
await row.click();
await guest.waitForURL(/\/profile\//);
// web shows profiles to guests: the name and standing, no actions.
await guest.getByRole('heading', { name: `ann${stamp}` }).waitFor({ timeout: 60_000 });
assert.equal(await guest.getByRole('button', { name: /Message|Vouch/ }).count(), 0);
await guest.screenshot({ path: `${out}/phone390-profile-guest.png` });
log('search finds a person by name, signed out');

assert.deepEqual(errors, [], 'no page errors');
log('all shell checks passed');
await browser.close();
