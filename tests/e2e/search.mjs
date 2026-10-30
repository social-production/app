// web's Search page kinds: a signed-out phone searches the test channel's handle and finds it
// first as a Channel, then filters to Channels, Communities and Events (the filter shows web's
// icons), and opens the channel from its result row.
//   node tests/e2e/search.mjs http://localhost:4174 [screens dir]
// It creates the shared test channel if it is missing (one founder sign-in), so it may run live.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, device, errors, log, TEST_CHANNEL, ensureTestChannel } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });

await ensureTestChannel();
const guest = await device({ viewport: { width: 390, height: 844 } });
const results = guest.locator('.stack .result-card');
const channelRow = results.filter({ has: guest.getByRole('heading', { name: TEST_CHANNEL, exact: true }) });

// A new channel takes a minute or two to reach the node's index: search until it lists it.
for (let i = 0; ; i++) {
  await guest.goto(`${BASE}/search?q=${TEST_CHANNEL}`);
  await guest.getByRole('heading', { name: 'Search', exact: true }).waitFor();
  if (await channelRow.first().waitFor({ timeout: 10_000 }).then(() => true, () => false)) break;
  if (i >= 15) throw new Error(`${TEST_CHANNEL} not found by search`);
}
// web-backend puts channels and communities first, an exact handle at the top.
assert.equal(await results.first().locator('h2').innerText(), TEST_CHANNEL);
assert.equal(await results.first().locator('.kind-chip').innerText(), 'Channel');
await guest.screenshot({ path: `${out}/phone390-search-kinds.png` });
log('search lists the channel first');

const filter = guest.getByRole('button', { name: 'Filter search' });
await filter.click();
const options = await guest.getByRole('menuitemradio').allInnerTexts();
assert.deepEqual(
  options.map((o) => o.replace('✓', '').trim()),
  ['All', 'Projects', 'Events', 'Help requests', 'Channels', 'Communities', 'Users'],
);
await guest.locator('.icon-menu-item .surface-icon').nth(5).waitFor({ timeout: 5_000 }); // web's option icons
await guest.getByRole('menuitemradio', { name: 'Communities' }).click();
await guest.waitForURL(/type=community/);
await channelRow.first().waitFor({ state: 'detached' });
for (const kind of await results.locator('.kind-chip').allInnerTexts()) assert.equal(kind, 'Community');
await filter.click();
await guest.getByRole('menuitemradio', { name: 'Channels' }).click();
await guest.waitForURL(/type=channel/);
await channelRow.first().waitFor();
for (const kind of await results.locator('.kind-chip').allInnerTexts()) assert.equal(kind, 'Channel');
log('the filter keeps one kind');

// Events: whatever the node lists, every row is an Event and opens /events/<id>.
await guest.goto(`${BASE}/search?q=e&type=event`);
await guest.getByRole('heading', { name: 'Search', exact: true }).waitFor();
await guest.waitForTimeout(3_000);
for (const kind of await results.locator('.kind-chip').allInnerTexts()) assert.equal(kind, 'Event');
for (const href of await results.evaluateAll((as) => as.map((a) => a.getAttribute('href'))))
  assert.match(href, /^\/events\//);
log(`events filter: ${await results.count()} event rows`);

await guest.goto(`${BASE}/search?q=${TEST_CHANNEL}&type=channel`);
await channelRow.first().click();
await guest.waitForURL(`**/channels/${TEST_CHANNEL}`);
await guest.getByRole('heading', { name: TEST_CHANNEL }).waitFor({ timeout: 60_000 });
log('a channel row opens its page');

assert.deepEqual(errors, []);
await browser.close();
log('search: ok');
