// M4b discussion threads end to end at phone width, against a local seed node started by
// scripts/dev-seed.sh. One person creates a project and starts a thread with a comment; another,
// who never joins, finds the thread in the Threads tab, upvotes it and replies to the comment.
//   node tests/e2e/threads.mjs http://localhost:4174 [screens dir]
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn } from './lib.mjs';

const [one, two] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-4);
const title = `Garden plots ${stamp}`;
const topic = `Where should the first beds go ${stamp}?`;
const first = `The corner plot gets the most sun ${stamp}`;
const reply = `Agreed, but it floods in spring ${stamp}`;

const ann = await device({ viewport: phone });
await phraseSignIn(ann, one.phrase.split(' '));
await ann.goto(`${BASE}/create/project`);
await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('Proposal description').fill('Shared beds for the street.');
for (let i = 0; i < 3; i++) await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByRole('button', { name: 'Publish project' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
const projectUrl = ann.url();
log('ann created the project');

await ann.getByRole('tab', { name: 'Threads' }).click();
await ann.getByRole('button', { name: 'New thread' }).click();
await ann.getByLabel('Thread title').fill(topic);
await ann.getByLabel('Opening post').fill('Corner plot or the fence strip?');
await ann.getByRole('button', { name: 'Publish thread' }).click();
await ann.getByRole('heading', { name: topic }).waitFor({ timeout: 60_000 });
const threadUrl = ann.url();
await ann.getByRole('textbox', { name: 'Write a comment...' }).fill(first);
await ann.getByRole('button', { name: 'Post comment' }).click();
await ann.locator('.comment-card').getByText(first).waitFor({ timeout: 30_000 });
log('ann started a thread and commented');

/** Reload `url` until `locator(page)` shows (gossip between the two hosted agents). */
async function waitFor(page, url, locator, what) {
  for (let i = 0; i < 30; i++) {
    await page.goto(url);
    try {
      await locator(page).waitFor({ timeout: 6_000 });
      return;
    } catch {
      // not gossiped yet
    }
  }
  throw new Error(`timed out waiting for ${what}`);
}

const bob = await device({ viewport: phone });
await phraseSignIn(bob, two.phrase.split(' '));
await bob.goto(projectUrl);
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
for (let i = 0; i < 30; i++) {
  await bob.getByRole('tab', { name: 'Threads' }).click();
  if (await bob.getByRole('link', { name: topic }).count()) break;
  await bob.waitForTimeout(4000);
  await bob.reload();
  await bob.getByRole('heading', { name: title }).waitFor({ timeout: 60_000 });
}
await bob.screenshot({ path: `${out}/phone390-threads.png` });
await bob.getByRole('link', { name: topic }).click();
await bob.getByRole('heading', { name: topic }).waitFor({ timeout: 60_000 });
await waitFor(bob, threadUrl, (p) => p.locator('.comment-card').getByText(first), "ann's comment");
await bob.locator('.overview-shell').getByRole('button', { name: 'Vote up' }).click();
await bob.locator('.comment-card').getByRole('button', { name: 'Reply' }).first().click();
await bob.getByRole('textbox', { name: 'Write a reply...' }).fill(reply);
await bob.getByRole('button', { name: 'Post reply' }).click();
await bob.locator('.reply-stack').getByText(reply).waitFor({ timeout: 30_000 });
log('bob upvoted and replied without joining');

await waitFor(ann, threadUrl, (p) => p.locator('.reply-stack').getByText(reply), "bob's reply");
const score = await ann.locator('.overview-shell .vote-count').innerText();
assert.equal(score.trim(), '1', "bob's upvote counts");
await ann.screenshot({ path: `${out}/phone390-thread.png`, fullPage: true });

assert.deepEqual(errors, [], 'no page errors');
log('all M4b thread checks passed');
await browser.close();
