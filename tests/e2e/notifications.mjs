// M4d notifications end to end at phone width (ADR 0046), against a local seed node started by
// scripts/dev-seed.sh (the dev founders, so a phase vote can pass). Ann starts a project, Bob
// joins and writes in its chat: Ann gets "Replied to your project". Ann replies: Bob gets
// "Replied to your comment". Ann proposes a phase move: Bob's card opens the Vote sheet, he votes
// it through and gets "Project phase change executed". The bell's count follows the reads.
//   node tests/e2e/notifications.mjs http://localhost:4174 [screens dir]
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn } from './lib.mjs';

const [one, two] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const title = `Seed swap ${stamp}`;
const hello = `I have tomato seeds ${stamp}`;
const answer = `Great, bring them ${stamp}`;

const composer = (page) => page.getByRole('textbox', { name: 'Message the project...' });
const bubble = (page, text) => page.locator('.chat-log').getByText(text).first();
// A copy shows "Sending…" until the write lands; leaving the page earlier would cancel it.
const sent = (page) =>
  page.locator('.chat-log').getByText('Sending…').waitFor({ state: 'detached', timeout: 60_000 });
const card = (page, text) => page.locator('.notification-card', { hasText: text }).first();
const bell = (page) => page.locator('.mobile-bottom-nav a[aria-label="Notifications"] .count-badge');

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
  const shown = await page.locator('.notifications-page').innerText();
  throw new Error(`timed out waiting for a "${text}" notification; page: ${shown}`);
}

const ann = await device({ viewport: phone });
await phraseSignIn(ann, one.phrase.split(' '));
const bob = await device({ viewport: phone });
await phraseSignIn(bob, two.phrase.split(' '));
// The first look on a device counts what came before as read; let both shells take it.
for (const page of [ann, bob]) {
  await page.goto(`${BASE}/notifications`);
  await page.getByRole('heading', { name: 'Notifications' }).waitFor({ timeout: 60_000 });
  await page.getByRole('button', { name: 'Mark all read' }).click();
}
log('both founders signed in');

await ann.goto(`${BASE}/create/project`);
await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('Proposal description').fill('Swap seeds every spring.');
for (let i = 0; i < 3; i++) await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByRole('button', { name: 'Publish project' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
const projectUrl = ann.url();
// Moving on from the Proposal phase needs support (web's signal gate), as in governance.mjs.
await ann.getByRole('button', { name: /^Support/ }).click();
log('ann created the project and supports it');

await bob.goto(projectUrl);
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await bob.getByRole('button', { name: 'Join project' }).click();
await bob.getByRole('button', { name: '2 members' }).waitFor({ timeout: 60_000 });
await bob.getByRole('button', { name: /^Support/ }).click();
await bob.getByRole('tab', { name: 'Chat' }).click();
await composer(bob).fill(hello);
await bob.getByRole('button', { name: 'Send', exact: true }).click();
await bubble(bob, hello).waitFor({ timeout: 30_000 });
await sent(bob);
log('bob joined and wrote in the chat');

const toAnn = await notified(ann, 'Replied to your project');
assert.match(await toAnn.innerText(), new RegExp(hello));
await bell(ann).waitFor({ timeout: 60_000 });
await ann.screenshot({ path: `${out}/phone390-notifications.png` });
await toAnn.click();
await ann.waitForURL(/\/projects\/[^?]+\?tab=chat&comment=/, { timeout: 60_000 });
await bubble(ann, hello).waitFor({ timeout: 60_000 });
await bubble(ann, hello).click();
await ann.getByRole('menuitem', { name: 'Reply' }).click();
await composer(ann).fill(answer);
await ann.getByRole('button', { name: 'Send', exact: true }).click();
await bubble(ann, answer).waitFor({ timeout: 30_000 });
await sent(ann);
await ann.goto(`${BASE}/notifications`);
await card(ann, 'Replied to your project').waitFor({ timeout: 60_000 });
assert.equal(await card(ann, 'Replied to your project').locator('.unread-dot').count(), 0);
log("ann's card opened the chat, where she replied; the card is read");

const toBob = await notified(bob, 'Replied to your comment');
assert.match(await toBob.innerText(), new RegExp(answer));
await toBob.getByRole('button', { name: 'Mark read' }).click();
await toBob.locator('.unread-dot').waitFor({ state: 'detached', timeout: 10_000 });
log('bob got the reply and marked it read');

// Reload until both supports have reached Ann (the gate unlocks "Advance to …").
const advance = () =>
  ann.getByRole('dialog', { name: 'Change phase' }).getByRole('button', { name: /^Advance to / });
for (let i = 0; ; i++) {
  await ann.goto(projectUrl);
  await ann.getByRole('tab', { name: 'Participation' }).click();
  await ann.getByRole('button', { name: 'Phase', exact: true }).click();
  if (await advance().isEnabled()) break;
  if (i >= 30) throw new Error('timed out waiting for the signal gate');
  await ann.waitForTimeout(3000);
}
await advance().click();
const sheet = ann.getByRole('dialog', { name: /^Advance to / });
await sheet.getByPlaceholder('State why this phase change should happen now.').fill('Ready to plan.');
await sheet.getByRole('button', { name: /^Advance to / }).click();
await sheet.waitFor({ state: 'hidden', timeout: 60_000 });
log('ann proposed a phase move');

const vote = await notified(bob, 'Vote on advancing to');
await bell(bob).waitFor({ timeout: 60_000 });
await vote.click();
await bob.waitForURL(/open=vote/, { timeout: 60_000 });
await bob.locator('.choice-bar .choice.yes').click({ timeout: 120_000 });
await bob.waitForURL((url) => !url.search.includes('open=vote'), { timeout: 60_000 });
// Leaving before the vote lands would cancel it; reloading the project also writes the Outcome.
for (let i = 0; !(await bob.locator('.feed-card-stage', { hasText: 'Production Plan' }).count()); i++) {
  if (i >= 40) throw new Error('timed out waiting for the phase move');
  await bob.waitForTimeout(3000);
  await bob.reload();
  await bob.locator('.project-detail h1').waitFor({ state: 'attached', timeout: 60_000 });
}
log("bob's card opened the Vote sheet and he voted yes; the project moved on");

const done = await notified(bob, 'Project phase change executed');
assert.match(await done.innerText(), /The project phase changed to /);
await bob.getByRole('button', { name: 'Mark all read' }).click();
await bell(bob).waitFor({ state: 'detached', timeout: 10_000 });
assert.equal(await bob.locator('.notification-card .unread-dot').count(), 0);
log('bob got the phase change; Mark all read cleared the bell');

assert.deepEqual(errors, [], 'no page errors');
log('all notification checks passed');
await browser.close();
