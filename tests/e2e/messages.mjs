// M4c direct and group chats end to end at phone width (ADR 0043). Two new accounts: Ann
// messages Bob from his profile, Bob sees the count on the send icon, finds the chat (unread) in
// Messages and replies (the count clears), then Ann starts a named group with Bob from the New
// message button.
//   node tests/e2e/messages.mjs http://localhost:4174 [screens dir]
// It signs up two test accounts, so it may run against the live test network too.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, device, errors, log, signUp } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const [annName, bobName] = [`ann${stamp}`, `bob${stamp}`];
const hello = `Hello Bob, want to swap seeds? ${stamp}`;
const answer = `Yes please, Ann ${stamp}`;
const group = `Garden crew ${stamp}`;
const kickoff = `Welcome to the crew ${stamp}`;

const bubble = (page, text) => page.locator('.chat-log').getByText(text).first();
const row = (page, name) => page.locator('.conversation-row', { hasText: name }).first();
const sendBadge = (page) =>
  page.locator('.mobile-bottom-nav a[aria-label="Messages"] .count-badge');

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

const ann = await device({ viewport: phone });
await signUp(ann, annName);
const bob = await device({ viewport: phone });
await signUp(bob, bobName);
// lib's keyOf opens Settings from the desktop rail; at phone width go there directly.
await bob.goto(`${BASE}/settings`);
const key = bob.locator('#settings-account .key');
await key.waitFor({ timeout: 60_000 });
const bobKey = (await key.innerText()).trim();
log('two accounts signed up');

await ann.goto(`${BASE}/profile/${bobKey}`);
await ann.getByRole('button', { name: `Message ${bobName}` }).click({ timeout: 120_000 });
await ann.getByRole('dialog').getByText(bobName).waitFor();
await ann.getByPlaceholder('Write a message…').fill(hello);
await ann.getByRole('dialog').getByRole('button', { name: 'Send' }).click();
await ann.waitForURL(/\/messages\?conversation=/, { timeout: 120_000 });
await bubble(ann, hello).waitFor({ timeout: 60_000 });
assert.equal(await ann.locator('.identity-title').innerText(), bobName, 'the header names Bob');
await ann.screenshot({ path: `${out}/phone390-dm.png` });
log('ann messaged bob from his profile');

// The shell counts unread chat messages every 30 s (web's unreadCounts.messages).
for (let i = 0; i < 12 && !(await sendBadge(bob).count()); i++) {
  await bob.goto(`${BASE}/`);
  await bob.waitForTimeout(10_000);
}
assert.equal(await sendBadge(bob).innerText(), '1', 'the send icon counts one unread message');
await bob.screenshot({ path: `${out}/phone390-send-badge.png` });
log('bob sees the count on the send icon');

const annRow = await rowShows(bob, annName);
assert.match(await annRow.locator('.unread-pill').innerText(), /^1$/, 'one unread message');
await bob.screenshot({ path: `${out}/phone390-messages-list.png` });
await annRow.click();
await bubble(bob, hello).waitFor({ timeout: 60_000 });
await bob.getByRole('textbox', { name: 'Write a message...' }).fill(answer);
await bob.getByRole('button', { name: 'Send', exact: true }).click();
await bubble(bob, answer).waitFor({ timeout: 30_000 });
log('bob read it and replied');

await bob.getByRole('button', { name: 'Back to chats' }).click();
await sendBadge(bob).waitFor({ state: 'detached', timeout: 30_000 });
log('reading it cleared the count');

await bubble(ann, answer).waitFor({ timeout: 90_000 });
log('ann sees the reply');

await ann.getByRole('button', { name: 'Back to chats' }).click();
await ann.getByRole('button', { name: 'New message' }).click();
const sheet = ann.getByRole('dialog');
await sheet.getByRole('tab', { name: 'Group' }).click();
await sheet.getByPlaceholder('Group name').fill(group);
await sheet.getByPlaceholder('Add members').fill(bobName);
await sheet.getByRole('option', { name: bobName }).click({ timeout: 60_000 });
await sheet.getByPlaceholder('Write a message…').fill(kickoff);
await ann.screenshot({ path: `${out}/phone390-compose-group.png` });
await sheet.getByRole('button', { name: 'Send' }).click();
await ann.waitForURL(/\/messages\?conversation=/, { timeout: 120_000 });
await ann.locator('.identity-title', { hasText: group }).waitFor({ timeout: 60_000 });
await bubble(ann, kickoff).waitFor({ timeout: 60_000 });
log('ann started a group');

await (await rowShows(bob, group)).click();
await bubble(bob, kickoff).waitFor({ timeout: 60_000 });
await bob.screenshot({ path: `${out}/phone390-group.png` });
log('bob is in the group');

assert.deepEqual(errors, [], 'no page errors');
log('all M4c messages checks passed');
await browser.close();
