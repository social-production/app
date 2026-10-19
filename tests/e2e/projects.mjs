// M2 projects end to end against a local seed node (run it as described in accounts.mjs):
// create through the "+" menu, find in the feed (archivist + index), join, support,
// search by tag, and the New-member limit of 2 projects a week.
//   node tests/e2e/projects.mjs http://localhost:4174
import assert from 'node:assert/strict';

import { BASE, browser, device, errors, log, signUp } from './lib.mjs';

const stamp = Date.now().toString(36).slice(-4);
const title = `Seed library ${stamp}`;

async function createProject(page, name, tag) {
  await page.goto(BASE);
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('menuitem', { name: /Project/ }).click();
  await page.getByRole('radio', { name: /Productive project/ }).click();
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByLabel('Title').fill(name);
  await page.getByLabel('Proposal description').fill('Share seeds between gardens.\n\nMore detail later.');
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByRole('radio', { name: /Online/ }).check();
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByPlaceholder('Type a tag, then Enter').fill(tag);
  await page.keyboard.press('Enter');
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByRole('button', { name: 'Publish project' }).click();
}

/** Reload the feed until `text` shows (the archivist indexes every 15 s, plus gossip). */
async function waitInFeed(page, text, path = '/') {
  const t0 = Date.now();
  for (;;) {
    await page.goto(`${BASE}${path}`);
    await page.locator('.feed-page').waitFor();
    await page.locator('.empty-card, .project-card').first().waitFor({ timeout: 60_000 });
    if (await page.locator('.project-card', { hasText: text }).count()) return (Date.now() - t0) / 1000;
    if (Date.now() - t0 > 180_000) throw new Error(`${text} not in the feed after 3 min`);
    await page.waitForTimeout(3000);
  }
}

const ann = await device();
await signUp(ann, `ann_${stamp}`);
await createProject(ann, title, 'seeds');
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await ann.getByRole('button', { name: 'Leave project' }).waitFor();
assert.equal((await ann.getByRole('button', { name: '1 members' }).count()), 1, 'creator is the first member');
log('created; the creator is a member');

const bob = await device();
await signUp(bob, `bob_${stamp}`);
log(`project in Bob's feed after ${(await waitInFeed(bob, title)).toFixed(0)}s`);
await bob.locator('.project-card', { hasText: title }).locator('a.title').click();
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await bob.getByRole('button', { name: 'Join project' }).click();
await bob.getByRole('button', { name: '2 members' }).waitFor({ timeout: 60_000 });
await bob.getByRole('button', { name: /^Support/ }).click();
await bob.getByText('100%').first().waitFor();
log('Bob joined and supported');

// A guest (signed out) browses the feed, as on web; supporting asks them to sign in.
const guest = await device();
await waitInFeed(guest, title);
const guestCard = guest.locator('.project-card', { hasText: title });
await guestCard.getByText(`ann_${stamp}`).waitFor();
await guestCard.getByRole('button', { name: /^Support/ }).click();
await guest.waitForURL(/\/onboarding$/);
log('a guest sees the feed with names, and Support asks them to sign in');

await ann.reload();
for (let i = 0; i < 20 && !(await ann.getByRole('button', { name: '2 members' }).count()); i++) {
  await ann.waitForTimeout(3000);
  await ann.reload();
}
await ann.getByRole('button', { name: '2 members' }).click();
await ann.locator('.people-sheet .person-row', { hasText: `bob_${stamp}` }).waitFor();
await ann.getByRole('button', { name: 'Close' }).last().click();
log('Ann sees Bob in the members sheet');

// Edits are proposals since M3: tests/e2e/governance.mjs covers them.
await waitInFeed(bob, title, `/search?q=seeds`);
log('search by tag finds the project');

await createProject(ann, `Second ${stamp}`, 'x');
await ann.getByRole('heading', { name: `Second ${stamp}` }).waitFor({ timeout: 120_000 });
await ann.goto(`${BASE}/create/project`);
await ann.getByText('(0 left)').waitFor({ timeout: 60_000 });
log('a New member sees 0 projects left after two');

assert.deepEqual(errors, [], 'no page errors');
log('all M2 project checks passed');
await browser.close();
