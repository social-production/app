// M5c help requests end to end (ADR 0067). Founder one asks for help with two roles (one
// place for a driver, any number of helpers); a new person opens it from the feed card, signs
// up as driver (which joins the request), sees the role full for others, and leaves again.
//   node tests/e2e/help-requests.mjs http://localhost:4174
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn, signUp } from './lib.mjs';

const [one] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const stamp = Date.now().toString(36).slice(-4);
const title = `Lift to the station ${stamp}`;

const ann = await device({ viewport: { width: 390, height: 844 } });
await phraseSignIn(ann, one.phrase.split(' '));
await ann.goto(BASE);
await ann.getByRole('button', { name: 'Create', exact: true }).click();
await ann.getByRole('menuitem', { name: /Help request/ }).click();
const next = () => ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByLabel(/^Title/).fill(title);
await ann.getByLabel('What do you need?').fill('Friday morning, with a suitcase.');
await next(); // → Location (physical)
await ann.getByLabel('Place').fill('Allotment gate');
await next(); // → Timing (two hours from now, for two hours)
await next(); // → Scope
await next(); // → Roles
await ann.getByLabel('Role title').fill('Driver');
await ann.getByLabel('Role details').fill('A car with a big boot.');
await ann.getByRole('button', { name: 'Add role' }).click();
await ann.getByLabel('Role title').nth(1).fill('Company');
await ann.getByLabel('Slots needed').nth(1).fill('0');
await next(); // → Overview
await ann.getByText('2 roles').waitFor();
await ann.getByRole('button', { name: 'Publish help request' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
assert.match(ann.url(), /\/help-requests\//);
const url = ann.url();
await ann.locator('.role-bar', { hasText: 'Driver' }).waitFor({ timeout: 120_000 });
await ann.getByText('Allotment gate').first().waitFor();
log('founder one asked for help');

const helper = await device();
await signUp(helper, `Helper${stamp}`);
// The feed card: when, where and sign-ups (the index reads new spaces within a minute).
for (let i = 0; ; i++) {
  await helper.goto(BASE);
  await helper.getByRole('button', { name: 'Sort public feed by' }).click();
  await helper.getByRole('menuitemradio', { name: 'Most recent' }).click();
  await helper.locator('.surface').first().waitFor({ timeout: 60_000 }).catch(() => {});
  const card = helper.locator('.help-request-card', { hasText: title });
  if (await card.count()) {
    await card.getByText('0 signed up · 1 needed').waitFor();
    break;
  }
  if (i >= 40) throw new Error('the help request never reached the feed');
  await helper.waitForTimeout(3000);
}
log('the feed shows its card');
await helper.goto(url);
const driver = helper.locator('.role-bar', { hasText: 'Driver' });
await driver.waitFor({ timeout: 120_000 });
await driver.getByRole('button', { name: 'Sign up' }).click();
await driver.getByRole('button', { name: 'Leave' }).waitFor({ timeout: 60_000 });
await driver.getByText('1 signed up · 1 needed').waitFor();
log('a helper signed up as driver');

for (let i = 0; ; i++) {
  await ann.goto(url);
  const full = ann.locator('.role-bar', { hasText: 'Driver' }).getByRole('button', { name: 'Full' });
  await ann.locator('.role-bar', { hasText: 'Driver' }).waitFor({ timeout: 120_000 });
  if (await full.count()) break;
  if (i >= 40) throw new Error('the driver role never showed as full');
  await ann.waitForTimeout(3000);
}
log('the asker sees the driver role full');

await driver.getByRole('button', { name: 'Leave' }).click();
await driver.getByRole('button', { name: 'Sign up' }).waitFor({ timeout: 60_000 });
log('the helper left the role');

assert.deepEqual(errors, []);
await browser.close();
log('help-requests: ok');
