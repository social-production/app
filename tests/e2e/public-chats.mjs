// M4c Public tab end to end at phone width (ADR 0044). Two new accounts: Ann starts a project,
// Bob writes in its chat without joining, Ann finds it (unread) in Messages → Public and
// replies there with the project's path (opening the members panel from the header), then Bob
// follows that link from the reply on the project's Chat tab and the unread count clears.
//   node tests/e2e/public-chats.mjs http://localhost:4174 [screens dir]
// It signs up two test accounts, so it may run against the live test network too.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, device, errors, log, signUp, continueSteps } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const title = `Tool library ${stamp}`;
const hello = `I can lend a drill ${stamp}`;
const answer = `Thanks, bring it Saturday ${stamp}`;

const bubble = (page, text) => page.locator('.chat-log').getByText(text).first();
// A copy shows "Sending…" until the write lands; leaving the page earlier would cancel it.
const sent = (page) =>
  page.locator('.chat-log').getByText('Sending…').waitFor({ state: 'detached', timeout: 60_000 });
const row = (page) => page.locator('.conversation-row', { hasText: title }).first();

/** The Public list polls every 8 s; reload until the project's row shows `unread` (0 = none). */
async function publicRow(page, unread) {
  for (let i = 0; i < 20; i++) {
    await page.goto(`${BASE}/messages`);
    await page.getByRole('tab', { name: /^Public/ }).click();
    for (let j = 0; j < 4; j++) {
      const r = row(page);
      if (await r.count()) {
        const pill = await r.locator('.unread-pill').count();
        const n = pill ? Number(await r.locator('.unread-pill').innerText()) : 0;
        if (n === unread) return r;
      }
      await page.waitForTimeout(3000);
    }
  }
  const shown = await page.locator('.conversation-list').innerText();
  throw new Error(`timed out waiting for the ${title} row with ${unread} unread; list: ${shown}`);
}

const ann = await device({ viewport: phone });
await signUp(ann, `ann${stamp}`);
const bob = await device({ viewport: phone });
await signUp(bob, `bob${stamp}`);
log('two accounts signed up');

await ann.goto(`${BASE}/create/project`);
await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('Proposal description').fill('Share tools between neighbours.');
await continueSteps(ann, 3);
await ann.getByRole('button', { name: 'Publish project' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
const projectUrl = ann.url();
log('ann created the project');

await bob.goto(projectUrl);
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await bob.getByRole('tab', { name: 'Chat' }).click();
await bob.getByRole('textbox', { name: 'Message the project...' }).fill(hello);
await bob.getByRole('button', { name: 'Send', exact: true }).click();
await bubble(bob, hello).waitFor({ timeout: 30_000 });
await sent(bob);
// An open Chat tab reads what arrives, so Bob leaves it before Ann replies.
await publicRow(bob, 0);
log('bob wrote in the project chat, which is now in his Public tab');

await (await publicRow(ann, 1)).click();
await ann.waitForURL(/\/messages\?project=/);
await bubble(ann, hello).waitFor({ timeout: 60_000 });
assert.match(await ann.locator('.identity-note').innerText(), /^Project chat · 1 member$/);
// web's members panel: the count opens the project's members.
await ann.getByRole('button', { name: '1 member', exact: true }).click();
await ann.locator('.people-sheet .person-row', { hasText: `ann${stamp}` }).waitFor();
await ann.screenshot({ path: `${out}/phone390-public-members.png` });
await ann.getByRole('button', { name: 'Close' }).last().click();
await ann.locator('.people-sheet').waitFor({ state: 'detached' });
// A path to one of the app's pages becomes a same-tab link; list previews drop it (web's
// linkifyMessageBody / withoutRawPaths).
const path = new URL(projectUrl).pathname;
await ann.getByRole('textbox', { name: 'Message the project...' }).fill(`${answer} ${path}`);
await ann.getByRole('button', { name: 'Send', exact: true }).click();
await bubble(ann, answer).waitFor({ timeout: 30_000 });
await sent(ann);
await ann.screenshot({ path: `${out}/phone390-public-chat.png` });
await ann.getByRole('button', { name: 'Back to chats' }).click();
const preview = await (await publicRow(ann, 0)).locator('.conversation-preview').innerText();
assert.equal(preview, answer, 'the preview drops the raw path');
await ann.screenshot({ path: `${out}/phone390-public-list.png` });
log('ann read and replied from Messages');

await publicRow(bob, 1);
await bob.goto(projectUrl);
await bob.getByRole('tab', { name: 'Chat' }).click();
await bubble(bob, answer).waitFor({ timeout: 90_000 });
const link = bob.locator(`.chat-log a[href="${path}"]`).first();
assert.equal(await link.getAttribute('target'), null, 'internal links open in the same tab');
await link.click();
await bob.waitForURL(projectUrl.split('?')[0] + '*');
await publicRow(bob, 0);
log("bob read ann's reply on the project page");

assert.deepEqual(errors, [], 'no page errors');
log('all Public tab checks passed');
await browser.close();
