// Photos and files in direct chats at phone width (ADR 0049), as web's LiveChatPanel sends
// them. Two new accounts: Ann messages Bob, then sends a photo and a 1.5 MB file (two parts)
// in one message; Bob sees "Photo" in his list, the photo in the bubble and in the viewer,
// and downloads the file intact.
//   node tests/e2e/attachments.mjs http://localhost:4174 [screens dir]
// It signs up two test accounts, so it may run against the live test network too.
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, signUp } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const [annName, bobName] = [`ann${stamp}`, `bob${stamp}`];
const hello = `Photos of the beds ${stamp}`;
// A 1×1 PNG: the composer resizes it to a JPEG, as web does.
const png = Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8DwHwAFBQIAX8jx0gAAAABJRU5ErkJggg==',
  'base64',
);
const notes = Buffer.alloc(1_500_000, `seed list ${stamp}\n`);

const row = (page, name) => page.locator('.conversation-row', { hasText: name }).first();
const lastBubble = (page) => page.locator('.chat-message').last();

/** Messages polls every 8 s; reload until a chat row for `name` shows. */
async function rowShows(page, name) {
  for (let i = 0; i < 20; i++) {
    await page.goto(`${BASE}/messages`);
    for (let j = 0; j < 4; j++) {
      if (await row(page, name).count()) return row(page, name);
      await page.waitForTimeout(3000);
    }
  }
  throw new Error(`timed out waiting for the chat with ${name}`);
}

/** The photo in `bubble` has loaded its bytes. */
async function photoLoaded(page, bubble) {
  const img = bubble.locator('.message-photo');
  await img.waitFor({ timeout: 60_000 });
  await page.waitForFunction((el) => el.complete && el.naturalWidth > 0, await img.elementHandle(), {
    timeout: 120_000,
  });
}

const ann = await device({ viewport: phone });
await signUp(ann, annName);
const bob = await device({ viewport: phone });
await signUp(bob, bobName);
await bob.goto(`${BASE}/settings`);
const key = bob.locator('#settings-account .key');
await key.waitFor({ timeout: 60_000 });
const bobKey = (await key.innerText()).trim();
log('two accounts signed up');

await ann.goto(`${BASE}/profile/${bobKey}`);
await ann.getByRole('button', { name: `Message ${bobName}` }).click({ timeout: 120_000 });
await ann.getByPlaceholder('Write a message…').fill(hello);
await ann.getByRole('dialog').getByRole('button', { name: 'Send' }).click();
await ann.waitForURL(/\/messages\?conversation=/, { timeout: 120_000 });
await ann.locator('.chat-log').getByText(hello).waitFor({ timeout: 60_000 });
log('ann messaged bob');

await ann.locator('input.attach-input[accept]').setInputFiles({
  name: 'bed.png',
  mimeType: 'image/png',
  buffer: png,
});
await ann.locator('.attach-preview img').waitFor();
await ann.locator('input.attach-input:not([accept])').setInputFiles({
  name: 'notes.txt',
  mimeType: 'text/plain',
  buffer: notes,
});
await ann.locator('.attach-preview', { hasText: 'notes.txt' }).waitFor();
await ann.screenshot({ path: `${out}/phone390-attach-previews.png` });
await ann.getByRole('button', { name: 'Send', exact: true }).click();
await ann.locator('.attach-preview').first().waitFor({ state: 'detached', timeout: 120_000 });
await ann.locator('.message-time', { hasText: 'Sending…' }).waitFor({ state: 'detached', timeout: 120_000 });
await photoLoaded(ann, lastBubble(ann));
assert.equal(await lastBubble(ann).locator('.message-file-name').innerText(), 'notes.txt');
assert.equal(await lastBubble(ann).locator('.message-file-size').innerText(), '1.4 MB');
await ann.screenshot({ path: `${out}/phone390-attach-sent.png` });
log('ann sent a photo and a file');

const annRow = await rowShows(bob, annName);
await annRow.getByText('Photo', { exact: true }).waitFor({ timeout: 90_000 });
await annRow.click();
await photoLoaded(bob, lastBubble(bob));
await bob.screenshot({ path: `${out}/phone390-attach-received.png` });
log('bob sees the photo');

await lastBubble(bob).locator('.photo-button').click();
await bob.locator('.photo-viewer img').waitFor();
await bob.screenshot({ path: `${out}/phone390-photo-viewer.png` });
await bob.keyboard.press('Escape');
await bob.locator('.photo-viewer').waitFor({ state: 'detached' });
log('the photo opens in the viewer and closes');

const [download] = await Promise.all([
  bob.waitForEvent('download', { timeout: 120_000 }),
  lastBubble(bob).locator('.message-file').click(),
]);
assert.equal(download.suggestedFilename(), 'notes.txt');
const saved = readFileSync(await download.path());
assert.ok(saved.equals(notes), 'the file arrives intact');
log('bob downloaded the file intact');

assert.deepEqual(errors, [], 'no page errors');
log('all attachment checks passed');
await browser.close();
