// A personal service's availability and bookings (ADR 0077) end to end at phone width. Ann
// creates a service with web's "Calendar booking" request mode, adds weekly hours and a one-off
// slot (tomorrow 18:00–19:00). Bob sees the open slots, taps the one-off row and requests it
// with its times locked. Ann accepts: the slot becomes her "Booked" booking, and Bob's row reads
// "Yours". A second request for the same time is refused: off-schedule requests are off.
//   node tests/e2e/bookings.mjs http://localhost:4174 [screens dir]
// It signs up two test accounts, so it may run against the live test network too.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, continueSteps, device, errors, log, signUp } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const title = `Piano lessons ${stamp}`;
const ask = `First lesson ${stamp}`;

const ann = await device({ viewport: phone });
await signUp(ann, `ann${stamp}`);
await ann.goto(`${BASE}/create/project`);
await ann.getByRole('radio', { name: /Personal service/ }).click();
await ann.getByRole('radio', { name: /Calendar booking/ }).click();
await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('Proposal description').fill('Beginner piano, one hour at a time.');
await continueSteps(ann, 3);
await ann.locator('.overview-row', { hasText: /Personal service\s*· Calendar booking/ }).waitFor();
await ann.getByRole('button', { name: 'Publish project' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
const projectUrl = ann.url().split('?')[0];
log('ann offers a calendar-booking service');

const participation = async (page) => {
  await page.goto(projectUrl);
  await page.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
  await page.getByRole('tab', { name: 'Participation' }).click();
};

/** Wait for `locator`, reloading the Participation tab while the page's reads catch up. */
async function until(page, locator, what) {
  for (let i = 0; i < 20; i++) {
    if (await locator.waitFor({ timeout: 10_000 }).then(() => true, () => false)) return;
    await participation(page);
  }
  throw new Error(`timed out waiting for ${what}`);
}

const button = (page, name) => page.getByRole('button', { name, exact: true });

await participation(ann);
await until(ann, button(ann, 'Add availability'), 'Add availability');
await button(ann, 'Add availability').click();
let sheet = ann.getByRole('dialog');
await sheet.getByRole('button', { name: 'Weekly', exact: true }).click();
await sheet.getByRole('button', { name: 'Add weekly hours' }).click();
await ann.locator('.rule-row', { hasText: 'Tuesday 09:00–12:00' }).waitFor({ timeout: 60_000 });
log('ann added weekly hours');

await button(ann, 'Add availability').click();
sheet = ann.getByRole('dialog');
await sheet.getByRole('button', { name: 'Add availability', exact: true }).click();
const available = ann.locator('.agenda-row', { hasText: 'Available' });
await until(ann, available, 'the one-off slot');
assert.match(await available.first().innerText(), /Open/);
await ann.screenshot({ path: `${out}/phone390-availability.png` });
log('ann added a one-off slot');

const bob = await device({ viewport: phone });
await signUp(bob, `bob${stamp}`);
await participation(bob);
await until(bob, bob.locator('.section-copy', { hasText: /open slots?/ }), 'the open slots');
const slot = bob.locator('.agenda-row', { hasText: 'Available' }).first();
await until(bob, slot, 'the one-off row');
await slot.click();
sheet = bob.getByRole('dialog');
assert.ok(await sheet.getByLabel('Start time').isDisabled(), "the slot's times are locked");
await sheet.getByPlaceholder('Request title').fill(ask);
await sheet.getByPlaceholder(/What do you need/).fill('Complete beginner.');
await sheet.getByRole('button', { name: 'Request', exact: true }).click();
await bob.getByRole('heading', { name: 'Your requests' }).waitFor({ timeout: 120_000 });
await bob.keyboard.press('Escape');
log('bob requested the slot');

await participation(ann);
await until(ann, button(ann, 'Requests'), 'Requests');
const card = ann.locator('.request-card-shell', { hasText: ask });
for (let i = 0; i < 10; i++) {
  await button(ann, 'Requests').click();
  if (await card.waitFor({ timeout: 15_000 }).then(() => true, () => false)) break;
  await participation(ann);
}
await card.locator('summary').click();
await card.getByRole('button', { name: 'Accept', exact: true }).click();
await card.getByText('Accepted', { exact: true }).waitFor({ timeout: 60_000 });
await ann.keyboard.press('Escape');
await until(ann, ann.locator('.agenda-row', { hasText: ask }), 'the booking');
assert.match(await ann.locator('.agenda-row', { hasText: ask }).innerText(), /Booked/);
await ann.locator('.rule-row').waitFor({ timeout: 60_000 });
// The weekly hours show on the creator's calendar too (web's slots for everyone).
await ann.locator('.calendar-cell[aria-label*="activit"]').nth(1).waitFor({ timeout: 60_000 });
await ann.screenshot({ path: `${out}/phone390-booked.png` });
log('ann accepted: the request is booked');

await participation(bob);
await until(bob, bob.locator('.agenda-row', { hasText: 'Yours' }), "bob's booking");
await bob.screenshot({ path: `${out}/phone390-your-booking.png` });
await button(bob, 'Request service').click();
sheet = bob.getByRole('dialog');
await sheet.getByPlaceholder('Request title').fill(`Second ${stamp}`);
await sheet.getByPlaceholder(/What do you need/).fill('Same time again.');
await sheet.getByRole('button', { name: 'Request', exact: true }).click();
await sheet.getByText(/outside the listed availability slots/).waitFor({ timeout: 30_000 });
log('bob sees his booking; an unlisted time is refused');

assert.deepEqual(errors, [], 'no page errors');
await browser.close();
log('bookings ok');
