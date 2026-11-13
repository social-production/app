// Service requests on a personal service (ADR 0076 part a) end to end at phone width. Ann offers
// a service; Bob opens its Participation tab, sends a request ("Request service") and finds web's
// request line in the direct chat it opened with Ann. Ann gets the svc-request card, sees the
// request in the rail's Requests and in her Requests sheet, and declines it with a reason; Bob's
// "Your requests" shows Declined with the reason. Bob sends a second one and cancels it. Last,
// Ann turns requests off in Request settings and Bob sees "Not taking requests." with no
// Request service button.
//   node tests/e2e/service-requests.mjs http://localhost:4174 [screens dir]
// It signs up two test accounts, so it may run against the live test network too.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, continueSteps, device, errors, log, signUp } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const title = `Bike repairs ${stamp}`;
const ask = `Fix my flat ${stamp}`;
const second = `Tune my gears ${stamp}`;

const ann = await device({ viewport: phone });
await signUp(ann, `ann${stamp}`);
await ann.goto(`${BASE}/create/project`);
await ann.getByRole('radio', { name: /Personal service/ }).click();
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('Proposal description').fill('I fix bikes on weekends.');
await continueSteps(ann, 3);
await ann.getByRole('button', { name: 'Publish proposal' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
const projectUrl = ann.url().split('?')[0];
log('ann offers a personal service');

const bob = await device({ viewport: phone });
await signUp(bob, `bob${stamp}`);

const participation = async (page) => {
  await page.goto(projectUrl);
  await page.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
  await page.getByRole('tab', { name: 'Participation' }).click();
};

/** Open a toolbar button, reloading while the page's reads catch up. */
async function press(page, name) {
  for (let i = 0; i < 10; i++) {
    const button = page.getByRole('button', { name, exact: true });
    if (await button.waitFor({ timeout: 15_000 }).then(() => true, () => false)) {
      await button.click();
      return;
    }
    await participation(page);
  }
  throw new Error(`no "${name}" button`);
}

async function sendRequest(page, what) {
  await press(page, 'Request service');
  const sheet = page.getByRole('dialog');
  await sheet.getByPlaceholder('Request title').fill(what);
  await sheet.getByPlaceholder(/What do you need/).fill('The back tyre keeps going flat.');
  await sheet.getByRole('button', { name: 'Request', exact: true }).click();
  await page.getByRole('heading', { name: 'Your requests' }).waitFor({ timeout: 120_000 });
  await page.locator('.request-card-shell', { hasText: what }).waitFor({ timeout: 60_000 });
}

await participation(bob);
assert.match(await bob.locator('.owner-note').innerText(), /Requests stay private/);
await sendRequest(bob, ask);
await bob.screenshot({ path: `${out}/phone390-your-requests.png` });
const mine = bob.locator('.request-card-shell', { hasText: ask });
await mine.locator('summary').click();
await mine.getByRole('link', { name: 'Open conversation' }).click();
await bob.waitForURL(/\/messages\?conversation=/, { timeout: 60_000 });
await bob.locator('.chat-log').getByText(`Request "${ask}" · Pending`).waitFor({ timeout: 60_000 });
log('bob sent a request; the chat with ann shows web\'s request line');

/** Reload until `locator` shows. */
async function until(page, url, locator, what) {
  for (let i = 0; i < 20; i++) {
    await page.goto(url);
    if (await locator.waitFor({ timeout: 10_000 }).then(() => true, () => false)) return;
  }
  throw new Error(`timed out waiting for ${what}`);
}

await until(
  ann,
  `${BASE}/notifications`,
  ann.locator('.notification-card', { hasText: `Request pending: ${ask}` }).first(),
  'the svc-request card',
);
log('ann got the svc-request card');

await participation(ann);
assert.match(await ann.locator('.owner-note').innerText(), /You run this service/);
await press(ann, 'Requests');
const card = ann.locator('.request-card-shell', { hasText: ask });
await card.waitFor({ timeout: 60_000 });
await card.locator('summary').click();
await card.getByRole('button', { name: 'Decline', exact: true }).click();
await card.getByPlaceholder('Reason (optional)').fill('Away that week');
await card.getByRole('button', { name: 'Decline request' }).click();
await card.getByText('Declined', { exact: true }).waitFor({ timeout: 60_000 });
await ann.screenshot({ path: `${out}/phone390-requests-declined.png` });
log('ann declined it with a reason');

await participation(bob);
await press(bob, 'Your requests');
const bobs = bob.locator('.request-card-shell', { hasText: ask });
await bobs.getByText('Declined', { exact: true }).waitFor({ timeout: 120_000 });
await bobs.locator('summary').click();
await bobs.getByText('Reason: Away that week').waitFor();
await bob.keyboard.press('Escape');
log('bob sees Declined and the reason');

await participation(bob);
await sendRequest(bob, second);
const tune = bob.locator('.request-card-shell', { hasText: second });
await tune.locator('summary').click();
await tune.getByRole('button', { name: 'Cancel request' }).click();
await tune.getByText('Cancelled', { exact: true }).waitFor({ timeout: 60_000 });
log('bob cancelled his second request');

await participation(ann);
await press(ann, 'Request settings');
await ann.getByRole('dialog').getByLabel('Enable requests').uncheck();
await ann.getByRole('dialog').getByRole('button', { name: 'Save settings' }).click();
await ann.locator('.requests-off-banner').waitFor({ timeout: 120_000 });
// The Overview tab keeps the Participation panel mounted but hidden: check what shows.
const banner = bob.locator('.requests-off-banner');
for (let i = 0; i < 20; i++) {
  await participation(bob);
  if (await banner.waitFor({ timeout: 5000 }).then(() => true, () => false)) break;
}
await banner.waitFor({ timeout: 10_000 });
assert.equal(await bob.getByRole('button', { name: 'Request service', exact: true }).count(), 0);
await bob.screenshot({ path: `${out}/phone390-not-taking-requests.png` });
log('ann turned requests off; bob sees "Not taking requests."');

assert.deepEqual(errors, [], 'no page errors');
await browser.close();
log('service requests ok');
