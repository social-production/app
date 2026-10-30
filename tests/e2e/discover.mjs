// web's Discover sheet (left rail, ADR 0068 part b): a signed-out phone opens the rail, taps
// Discover, sees the test channel, filters to Communities and back, searches, and opens it.
//   node tests/e2e/discover.mjs http://localhost:4174 [screens dir]
// It creates the shared test channel if it is missing (one founder sign-in), so it may run live.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, device, errors, log, TEST_CHANNEL, ensureTestChannel } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });

await ensureTestChannel();
const guest = await device({ viewport: { width: 390, height: 844 } });
await guest.goto(BASE);
const sheet = guest.locator('.scope-discover');
const row = sheet.locator('.scope-row', { hasText: TEST_CHANNEL });
// A new channel takes a minute or two to reach the node's index: reopen until it lists it.
for (let i = 0; ; i++) {
  await guest.getByRole('button', { name: 'Open networks' }).click();
  await guest.getByRole('button', { name: 'Discover' }).click();
  await sheet.getByRole('heading', { name: 'Discover' }).waitFor();
  if (await row.first().waitFor({ timeout: 10_000 }).then(() => true, () => false)) break;
  if (i >= 20) throw new Error(`${TEST_CHANNEL} never appeared in Discover`);
  await guest.reload();
}
assert.match(await row.first().locator('.scope-meta').innerText(), /^\d+ members?$/);
await guest.screenshot({ path: `${out}/phone390-discover.png` });
log('Discover lists the test channel');

const filter = sheet.getByRole('button', { name: 'Filter channels and communities' });
await filter.click();
await guest.getByRole('menuitemradio', { name: 'Communities' }).click();
await row.first().waitFor({ state: 'detached' });
await filter.click();
await guest.getByRole('menuitemradio', { name: 'Channels' }).click();
await sheet.getByPlaceholder('Search channels').fill(TEST_CHANNEL);
await sheet.getByPlaceholder('Search channels').fill(`${TEST_CHANNEL}-nothing-here`);
await sheet.getByText('No matches.').waitFor({ timeout: 10_000 });
await sheet.getByPlaceholder('Search channels').fill(TEST_CHANNEL);
await row.first().click();
await guest.waitForURL(`**/channels/${TEST_CHANNEL}`);
await guest.getByRole('heading', { name: TEST_CHANNEL }).waitFor({ timeout: 60_000 });
assert.equal(await sheet.count(), 0, 'the sheet closed');
assert.equal(await guest.locator('.left-rail[data-open="true"]').count(), 0, 'the rail closed');
log('Discover filters, searches and opens the channel');

assert.deepEqual(errors, []);
await browser.close();
log('discover: ok');
