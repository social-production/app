// M5c private events end to end (ADR 0061). A new person signs up; founder one creates an
// invite-only, organizer-controlled event inviting them, with its plan up front, so the event
// opens in Activity. The invitee finds it under Invites (left rail and notification), accepts
// and lands on the event.
//   node tests/e2e/private-events.mjs http://localhost:4174
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn, signUp } from './lib.mjs';

const [one] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const stamp = Date.now().toString(36).slice(-4);
const guestName = `Guest${stamp}`;
const title = `Birthday lunch ${stamp}`;

const guest = await device({ viewport: { width: 390, height: 844 } });
await signUp(guest, guestName);
log(`${guestName} signed up`);

const ann = await device();
await phraseSignIn(ann, one.phrase.split(' '));
await ann.goto(BASE);
await ann.getByRole('button', { name: 'Create', exact: true }).click();
await ann.getByRole('menuitem', { name: /Event/ }).click();
const next = () => ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByLabel('Event title').fill(title);
await ann.getByLabel('Description').fill('Lunch, then cake.');
await next(); // → Audience
await ann.getByText('Invite only', { exact: true }).click();
await next(); // → Who
// The directory may take a moment to list someone who just signed up.
const invite = ann.getByLabel('Invite people');
for (let i = 0; ; i++) {
  await invite.fill(guestName);
  const row = ann.getByRole('option', { name: guestName });
  if (await row.count()) {
    await row.click();
    break;
  }
  if (i >= 30) throw new Error(`${guestName} never appeared in the people search`);
  await invite.fill('');
  await ann.waitForTimeout(3000);
}
await ann.locator('.selected-chip', { hasText: guestName }).waitFor();
await next(); // → Control (organizer-controlled is web's preselection)
await ann.getByText('Current selection:').waitFor();
await next(); // → Location (decide later)
await next(); // → Plan
const day = new Date(Date.now() + 10 * 24 * 60 * 60 * 1000);
const iso = `${day.getFullYear()}-${String(day.getMonth() + 1).padStart(2, '0')}-${String(day.getDate()).padStart(2, '0')}`;
await ann.getByLabel('Plan title').fill('Lunch at the hall');
await ann.getByLabel('Plan description').fill('Everyone brings a plate.');
await ann.getByLabel(/^Date/).fill(iso);
await ann.getByLabel('Start time').fill('12:00');
await ann.getByLabel('Finish time').fill('15:00');
await ann.locator('.stage-card').getByLabel('Title').fill('Lunch');
await ann.locator('.stage-card').getByLabel('Details').fill('Plates on the long table.');
await next(); // → Overview
await ann.getByText('Organizer-controlled').waitFor();
await ann.getByRole('button', { name: 'Publish event' }).click();
try {
  await ann.getByRole('heading', { name: title }).waitFor({ timeout: 180_000 });
} catch (e) {
  // Say why publishing stopped (the create sheet's status line) before failing.
  log('status:', await ann.locator('.status-note').allInnerTexts(), 'errors:', errors);
  await ann.screenshot({ path: 'private-events-fail.png', fullPage: true });
  throw e;
}
assert.match(ann.url(), /\/events\/[^/]+\.organized\./);
await ann.locator('.feed-card-top .private-mark').waitFor();
await ann.locator('.feed-card-stage', { hasText: 'Activity' }).waitFor({ timeout: 60_000 });
log('founder one created the private event; it opened in Activity');

// The invitee: the left rail's Invites row (web's count), then Accept.
const rows = () => guest.getByRole('link', { name: /^Invites/ });
for (let i = 0; ; i++) {
  await guest.goto(`${BASE}/invites`);
  // The first read joins the event's network, so let the page finish loading.
  await guest
    .getByText(new RegExp(`No invites waiting|${title}`))
    .first()
    .waitFor({ timeout: 180_000 });
  if (await guest.getByText(title).count()) break;
  if (i >= 40) {
    log('invites page:', await guest.locator('.invites').innerText(), 'errors:', errors);
    throw new Error('the invite never arrived');
  }
  await guest.waitForTimeout(3000);
}
// "· from <name>" follows once the inviter has a profile name.
await guest.locator('.invites .kind', { hasText: 'Private event' }).waitFor();
log('the invite arrived');
await guest.getByRole('button', { name: 'Accept' }).click();
await guest.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
assert.match(guest.url(), /\.organized\./);
await guest.locator('.feed-card-top .private-mark').waitFor();
await guest.goto(`${BASE}/invites`);
await guest.getByText('No invites waiting.').waitFor({ timeout: 60_000 });
assert.equal(await rows().count(), 0, 'no Invites row once answered');
log('the invitee accepted and joined');

assert.deepEqual(errors, []);
await browser.close();
log('private-events: ok');
