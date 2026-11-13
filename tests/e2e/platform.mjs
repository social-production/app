// The Platform channel and its moderators (ADR 0095), and deactivating an account (ADR 0088).
// Founder two follows /platform (the first follow writes the fixed channel), opens Moderators,
// volunteers (with nobody active last week the quorum is 0, so they are seated at once, as on
// web), then steps down. A new person sees the board from the rail's link without Volunteer
// (New standing), deactivates their account from Settings, and can't sign in again.
//   node tests/e2e/platform.mjs http://localhost:4174 [screenshot dir]
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, onboarding, phraseSignIn, signUp } from './lib.mjs';

const [, two] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const stamp = Date.now().toString(36).slice(-4);
const shots = process.argv[3];

const ann = await device({ viewport: { width: 390, height: 844 } });
await phraseSignIn(ann, two.phrase.split(' '));
await ann.goto(`${BASE}/platform`);
await ann.getByRole('heading', { name: 'Platform' }).waitFor({ timeout: 60_000 });
await ann.getByText('The channel for the whole network.').waitFor();
const leave = ann.getByRole('button', { name: 'Leave platform' });
if (!(await leave.count())) await ann.getByRole('button', { name: 'Join platform' }).click();
await leave.waitFor({ timeout: 120_000 });
log('founder two follows the Platform channel');

await ann.getByRole('button', { name: 'Moderators' }).click();
const sheet = ann.locator('.platform-board .overlay-sheet, .platform-board').first();
await ann.getByRole('heading', { name: 'Moderators' }).waitFor();
const volunteer = ann.getByRole('button', { name: 'Volunteer as moderator' });
await volunteer.waitFor({ timeout: 60_000 });
await volunteer.click();
await ann.getByText('You are now seeking a moderator role.').waitFor({ timeout: 60_000 });
await sheet.getByText('Active moderators').waitFor({ timeout: 60_000 });
await sheet.getByText('Dev Founder Two').first().waitFor();
if (shots) await ann.screenshot({ path: `${shots}/moderators.png` });
log('volunteering seats them while the quorum is 0');

await ann.getByRole('button', { name: 'Step down as moderator' }).click();
await ann.getByRole('button', { name: 'Confirm step down' }).click();
await ann.getByText('You have stepped down as moderator.').waitFor({ timeout: 60_000 });
await volunteer.waitFor({ timeout: 60_000 });
log('stepping down leaves the board');

const bob = await device({ viewport: { width: 390, height: 844 } });
const name = `Leaver${stamp}`;
const words = await signUp(bob, name);
await bob.goto(`${BASE}/platform?board=1`);
await bob.getByRole('heading', { name: 'Moderators' }).waitFor({ timeout: 60_000 });
await bob.getByText(/No moderators listed yet|Seeking position|Active moderators/).first().waitFor({
  timeout: 60_000,
});
assert.equal(await bob.getByRole('button', { name: 'Volunteer as moderator' }).count(), 0);
log('a New member sees the board without Volunteer');

await bob.goto(`${BASE}/settings`);
await bob.getByLabel('Type your name to confirm').fill(name);
await bob.getByRole('button', { name: 'Deactivate account' }).click();
await bob.locator('.shell[data-signed-in="false"]').waitFor({ timeout: 60_000 });
log('deactivating signs them out');

await onboarding(bob);
await bob.getByRole('tab', { name: 'Log in' }).click();
await bob.getByLabel('Recovery phrase').fill(words.join(' '));
await bob.getByRole('button', { name: 'Log in', exact: true }).click();
await bob.getByText('This account was deactivated, so it can no longer sign in.').waitFor({
  timeout: 120_000,
});
await bob.locator('.shell[data-signed-in="false"]').waitFor({ timeout: 60_000 });
log('a deactivated account cannot sign in again');

assert.deepEqual(errors, []);
await browser.close();
log('platform: ok');
