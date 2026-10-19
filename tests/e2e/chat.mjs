// M4a project chat end to end at phone width, against a local seed node started by
// scripts/dev-seed.sh. One person creates a project; another, who never joins, posts in its
// chat; the creator replies; the first message is deleted and the reply then quotes it as deleted.
//   node tests/e2e/chat.mjs http://localhost:4174 [screens dir]
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn } from './lib.mjs';

const [one, two] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-4);
const title = `Seed swap ${stamp}`;
const hello = `Count me in for the seed swap ${stamp}`;
const answer = `Great, bring tomato seeds ${stamp}`;

const ann = await device({ viewport: phone });
await phraseSignIn(ann, one.phrase.split(' '));
await ann.goto(`${BASE}/create/project`);
await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('Proposal description').fill('Swap seeds every spring.');
for (let i = 0; i < 3; i++) await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByRole('button', { name: 'Publish project' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
const projectUrl = ann.url();
log('ann created the project');

const composer = (page) => page.getByRole('textbox', { name: 'Message the project...' });
const bubble = (page, text) => page.locator('.chat-log').getByText(text, { exact: true });

/** Open the Chat tab, then wait (polls run every 5 s while it's open) until `text` shows. */
async function chatShows(page, text, what) {
  for (let i = 0; i < 30; i++) {
    await page.locator('.project-detail h1').waitFor({ timeout: 60_000 });
    await page.getByRole('tab', { name: 'Chat' }).click();
    for (let j = 0; j < 4; j++) {
      if (await page.locator('.chat-log').getByText(text).count()) return;
      await page.waitForTimeout(3000);
    }
    await page.reload();
  }
  throw new Error(`timed out waiting for ${what}`);
}

const bob = await device({ viewport: phone });
await phraseSignIn(bob, two.phrase.split(' '));
await bob.goto(projectUrl);
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await bob.getByRole('tab', { name: 'Chat' }).click();
await composer(bob).fill(hello);
await bob.getByRole('button', { name: 'Send', exact: true }).click();
await bubble(bob, hello).waitFor({ timeout: 30_000 });
assert.equal(await bob.getByRole('button', { name: 'Join project' }).count() > 0, true, 'bob never joined');
log('bob posted without joining');

await ann.reload();
await chatShows(ann, hello, "bob's message");
await bubble(ann, hello).click();
await ann.getByRole('menuitem', { name: 'Reply' }).click();
await composer(ann).fill(answer);
await ann.getByRole('button', { name: 'Send', exact: true }).click();
await bubble(ann, answer).waitFor({ timeout: 30_000 });
await ann.screenshot({ path: `${out}/phone390-chat.png` });
log('ann replied');

await bob.reload();
await chatShows(bob, answer, "ann's reply");
await bubble(bob, hello).click();
await bob.getByRole('menuitem', { name: 'Delete' }).click();
await bob.getByRole('alertdialog').getByRole('button', { name: 'Delete' }).click();
await bubble(bob, hello).waitFor({ state: 'detached', timeout: 30_000 });
log('bob deleted his message');

await ann.reload();
await chatShows(ann, 'Deleted message', 'the deleted quote');
assert.equal(await bubble(ann, hello).count(), 0, 'the deleted message is gone for ann too');
await ann.screenshot({ path: `${out}/phone390-chat-deleted.png` });

assert.deepEqual(errors, [], 'no page errors');
log('all M4a chat checks passed');
await browser.close();
