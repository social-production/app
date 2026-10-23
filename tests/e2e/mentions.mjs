// @mentions in a project chat end to end at phone width (ADR 0047). Two new accounts: Bob
// writes in Ann's project chat; Ann types "@bo…", picks Bob from web's mention menu and sends.
// The bubble links the name to Bob's profile. Bob gets "You were mentioned.", whose card opens
// the chat on that message, marked until he writes.
//   node tests/e2e/mentions.mjs http://localhost:4174 [screens dir]
// It signs up two test accounts, so it may run against the live test network too.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, device, errors, keyOf, log, signUp } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const title = `Bike repair ${stamp}`;
const bobName = `bob${stamp}`;
const hello = `I can fix brakes ${stamp}`;
const callOut = `can you bring a pump? ${stamp}`;
const thanks = `Sure ${stamp}`;

const composer = (page) => page.getByRole('textbox', { name: 'Message the project...' });
const bubble = (page, text) => page.locator('.chat-message', { hasText: text }).first();
// A copy shows "Sending…" until the write lands; leaving the page earlier would cancel it.
const sent = (page) =>
  page.locator('.chat-log').getByText('Sending…').waitFor({ state: 'detached', timeout: 60_000 });
const card = (page, text) => page.locator('.notification-card', { hasText: text }).first();

async function send(page, text) {
  await composer(page).fill(text);
  await page.getByRole('button', { name: 'Send', exact: true }).click();
  await bubble(page, text).waitFor({ timeout: 30_000 });
  await sent(page);
}

/** Reload Notifications until a card with `text` shows (the list is read from the network). */
async function notified(page, text) {
  for (let i = 0; i < 40; i++) {
    await page.goto(`${BASE}/notifications`);
    await page.getByRole('heading', { name: 'Notifications' }).waitFor({ timeout: 60_000 });
    for (let j = 0; j < 3; j++) {
      if (await card(page, text).count()) return card(page, text);
      await page.waitForTimeout(3000);
    }
  }
  throw new Error(`timed out waiting for a "${text}" notification`);
}

const ann = await device({ viewport: phone });
await signUp(ann, `ann${stamp}`);
const bob = await device({ viewport: phone });
await signUp(bob, bobName);
const bobKey = await keyOf(bob);
// The first look on a device counts what came before as read, so Bob takes it now.
await bob.goto(`${BASE}/notifications`);
await bob.getByRole('heading', { name: 'Notifications' }).waitFor({ timeout: 60_000 });
log('two accounts signed up');

await ann.goto(`${BASE}/create/project`);
await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('Proposal description').fill('Fix bikes together on Sundays.');
for (let i = 0; i < 3; i++) await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByRole('button', { name: 'Publish project' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
const projectUrl = ann.url();
log('ann created the project');

await bob.goto(projectUrl);
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await bob.getByRole('tab', { name: 'Chat' }).click();
await send(bob, hello);
log('bob wrote in the project chat');

// The directory can lag behind a new account: reload until the menu offers Bob.
const option = ann.getByRole('option', { name: bobName });
for (let i = 0; ; i++) {
  await ann.goto(`${projectUrl}?tab=chat`);
  await bubble(ann, hello).waitFor({ timeout: 90_000 });
  await composer(ann).fill('');
  await composer(ann).pressSequentially(`@${bobName.slice(0, 5)}`, { delay: 30 });
  try {
    await option.waitFor({ timeout: 10_000 });
    break;
  } catch (e) {
    if (i >= 12) throw e;
  }
}
await ann.screenshot({ path: `${out}/phone390-mention-menu.png` });
await option.click();
assert.equal(await composer(ann).inputValue(), `@${bobName} `);
await composer(ann).pressSequentially(callOut);
await ann.getByRole('button', { name: 'Send', exact: true }).click();
await bubble(ann, callOut).waitFor({ timeout: 30_000 });
await sent(ann);
const link = bubble(ann, callOut).locator('a.mention');
assert.equal(await link.innerText(), `@${bobName}`);
assert.match(await link.getAttribute('href'), new RegExp(`/profile/${bobKey}$`));
log('ann mentioned bob from the menu; the bubble links to his profile');

const mention = await notified(bob, 'You were mentioned.');
assert.match(await mention.innerText(), new RegExp(title));
await mention.click();
await bob.waitForURL(/\/projects\/[^?]+\?tab=chat&comment=/, { timeout: 60_000 });
await bubble(bob, callOut).waitFor({ timeout: 60_000 });
await bob.locator('.chat-message.highlighted', { hasText: callOut }).waitFor({ timeout: 10_000 });
assert.ok(await bubble(bob, callOut).isVisible());
await bob.screenshot({ path: `${out}/phone390-mention-linked.png` });
log("bob's card opened the chat on the message, marked");

await send(bob, thanks);
await bob.waitForURL((url) => !url.search.includes('comment='), { timeout: 10_000 });
assert.equal(await bob.locator('.chat-message.highlighted').count(), 0);
log('writing in the chat dropped the mark and the link');

assert.deepEqual(errors, [], 'no page errors');
log('all mention checks passed');
await browser.close();
