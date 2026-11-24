// A channel's picture and cover (ADR 0109, Channel Pictures mockups). Founder one creates a
// channel with a picture and cover (no vote); founder two joins and proposes a new cover from
// Settings; one sees Now › Proposed on the Vote sheet and votes it through; both pages then
// show the new cover. Screenshots at 390px.
//   node tests/e2e/scope-pictures.mjs http://localhost:4174 [screens dir]
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync } from 'node:fs';
import { deflateSync } from 'node:zlib';

import { BASE, browser, device, errors, log, phraseSignIn } from './lib.mjs';
import { keepCrop } from './demo/art.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const handle = `pictures${Date.now().toString(36).slice(-5)}`;
const [one, two] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));

/** A solid-colour PNG, `w`×`h`, for the file inputs. */
function png(w, h, [r, g, b]) {
  const crcTable = Array.from({ length: 256 }, (_, n) => {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    return c >>> 0;
  });
  const crc = (buf) => {
    let c = 0xffffffff;
    for (const byte of buf) c = crcTable[(c ^ byte) & 0xff] ^ (c >>> 8);
    return (c ^ 0xffffffff) >>> 0;
  };
  const chunk = (type, data) => {
    const len = Buffer.alloc(4);
    len.writeUInt32BE(data.length);
    const body = Buffer.concat([Buffer.from(type), data]);
    const sum = Buffer.alloc(4);
    sum.writeUInt32BE(crc(body));
    return Buffer.concat([len, body, sum]);
  };
  const header = Buffer.alloc(13);
  header.writeUInt32BE(w, 0);
  header.writeUInt32BE(h, 4);
  header.set([8, 2, 0, 0, 0], 8);
  const row = Buffer.concat([Buffer.from([0]), Buffer.alloc(w * 3).map((_, i) => [r, g, b][i % 3])]);
  const raw = Buffer.concat(Array.from({ length: h }, () => row));
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', header),
    chunk('IDAT', deflateSync(raw)),
    chunk('IEND', Buffer.alloc(0)),
  ]);
}
const file = (name, w, h, colour) => ({ name, mimeType: 'image/png', buffer: png(w, h, colour) });

/** The cover's image hash (blob URLs differ on every load). */
const coverOf = (page) => page.locator('.scope-hero .scope-cover').getAttribute('data-cover');

// One creates the channel with a picture and a cover.
const a = await device({ viewport: phone });
await phraseSignIn(a, one.phrase.split(' '));
await a.goto(`${BASE}/create/channel`);
await a.getByLabel('Picture image').setInputFiles(file('picture.png', 200, 200, [220, 120, 40]));
await keepCrop(a);
await a.getByLabel('Cover image').setInputFiles(file('cover.png', 600, 200, [60, 140, 70]));
await keepCrop(a);
await a.locator('.scope-sheet-art .art-new').nth(1).waitFor({ timeout: 60_000 });
await a.getByLabel(/Channel handle/).fill(handle);
await a.getByLabel(/Description/).fill('Made by the pictures test.');
await a.screenshot({ path: `${out}/phone390-scope-create-art.png` });
await a.getByRole('button', { name: 'Create channel' }).click();
await a.getByRole('heading', { name: handle }).waitFor({ timeout: 120_000 });
await a.locator('.scope-hero .scope-avatar img').waitFor({ timeout: 60_000 });
const firstCover = await coverOf(a);
assert.ok(firstCover, 'the cover is set');
assert.match(
  await a.locator('.scope-hero .scope-cover').evaluate((el) => getComputedStyle(el).backgroundImage),
  /^url\(/,
  'the cover shows',
);
const url = a.url();
await a.screenshot({ path: `${out}/phone390-scope-art-page.png` });
log('one made the channel with a picture and cover');

// Two joins and proposes a new cover.
const b = await device({ viewport: phone });
await phraseSignIn(b, two.phrase.split(' '));
const heading = b.getByRole('heading', { name: handle });
for (let i = 0; ; i++) {
  await b.goto(url);
  if (await heading.waitFor({ timeout: 15_000 }).then(() => true, () => false)) break;
  if (i >= 20) throw new Error('the channel never reached the index');
}
await b.getByRole('button', { name: `Join ${handle}` }).click();
await b.getByRole('button', { name: `Leave ${handle}` }).waitFor({ timeout: 120_000 });
await b.getByRole('button', { name: 'Settings', exact: true }).click();
const settings = b.locator('.scope-settings');
await settings.getByLabel('Cover image').setInputFiles(file('new-cover.png', 600, 200, [40, 90, 200]));
await keepCrop(b);
await settings.locator('.scope-cover .art-new').waitFor({ timeout: 60_000 });
await b.screenshot({ path: `${out}/phone390-scope-art-settings.png` });
await settings.getByRole('button', { name: 'Propose change' }).click();
await settings.locator('.decision-card.open', { hasText: 'New cover' }).waitFor({ timeout: 60_000 });
await settings.locator('.sheet-close').click();
log('two proposed a new cover');

// One votes from the Vote sheet, which shows Now and Proposed.
const fab = a.getByRole('button', { name: /^Vote: \d+ waiting/ });
for (let i = 0; ; i++) {
  await a.goto(url);
  await a.getByRole('heading', { name: handle }).waitFor({ timeout: 60_000 });
  if (await fab.waitFor({ timeout: 15_000 }).then(() => true, () => false)) break;
  if (i >= 20) throw new Error('the Vote button never showed');
}
await fab.click();
const vote = a.locator('.scope-vote');
await vote.getByText('New cover').waitFor();
await vote.locator('.decision-art figcaption', { hasText: 'Now' }).waitFor();
await vote.locator('.decision-art figcaption', { hasText: 'Proposed' }).waitFor();
await a.waitForTimeout(1500);
await a.screenshot({ path: `${out}/phone390-scope-art-vote.png` });
await vote.getByRole('button', { name: 'Yes', exact: true }).click();
// Stay until the vote is written (leaving at once can drop it).
await vote.getByText(/You voted Yes/).waitFor({ timeout: 60_000 });
await vote.waitFor({ state: 'detached', timeout: 30_000 });
log('one voted');

for (let i = 0; ; i++) {
  await a.goto(url);
  await a.getByRole('heading', { name: handle }).waitFor({ timeout: 60_000 });
  await a.waitForTimeout(2000);
  if ((await coverOf(a)) !== firstCover) break;
  if (i >= 20) throw new Error('the new cover never showed');
}
await a.screenshot({ path: `${out}/phone390-scope-art-after.png` });
log('the new cover shows');

assert.deepEqual(errors, [], 'no page errors');
log('scope-pictures: ok');
await browser.close();
