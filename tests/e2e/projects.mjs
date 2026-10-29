// M2 projects end to end against a local seed node (run it as described in accounts.mjs):
// create a channel and a project tagged with it through the "+" menu, find it in the feed
// (archivist + index) and on the channel's page (ADR 0068), join, support, search by tag,
// web's tag chips and their "+N" sheet, and the New-member limit of 2 listings a week (a
// channel counts until the next directory reset, ADR 0068).
//   node tests/e2e/projects.mjs http://localhost:4174
import assert from 'node:assert/strict';

import { BASE, browser, device, errors, log, pickScopeTag, signUp, tagTestChannel } from './lib.mjs';

const stamp = Date.now().toString(36).slice(-4);
const title = `Seed library ${stamp}`;
const channel = `seeds${stamp}`;

async function createChannel(page, handle) {
  await page.goto(BASE);
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('menuitem', { name: /Channel/ }).click();
  await page.getByLabel(/Channel handle/).fill(handle);
  await page.getByLabel(/Description/).fill('Seed saving and swaps.');
  await page.getByRole('button', { name: 'Create channel' }).click();
  await page.getByRole('heading', { name: handle }).waitFor({ timeout: 120_000 });
}

// Starts from the page you are on: right after making the channel, it is offered from this
// device's memory before the index lists it (a reload would wait on the index, ~1–3 min live).
async function createProject(page, name, tag) {
  await page.getByRole('button', { name: 'Create', exact: true }).click();
  await page.getByRole('menuitem', { name: /Project/ }).click();
  await page.getByRole('radio', { name: /Productive project/ }).click();
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByLabel('Title').fill(name);
  await page.getByLabel('Proposal description').fill('Share seeds between gardens.\n\nMore detail later.');
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByRole('radio', { name: /Online/ }).check();
  await page.getByRole('button', { name: 'Continue' }).click();
  // web's scope step: the primary channel tag, then the shared test channel as a second one
  // (so the card shows web's "+1" on phones).
  await pickScopeTag(page, 'Primary channel tag', tag);
  await page.getByText('Add more channels or communities').click();
  await tagTestChannel(page, 'Additional channel tags');
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
    // "Loading feed..." is an .empty-card too; count only once the feed has loaded.
    await page.getByText('Loading feed...').waitFor({ state: 'detached', timeout: 60_000 });
    if (await page.locator('.project-card', { hasText: text }).count()) return (Date.now() - t0) / 1000;
    if (Date.now() - t0 > 180_000) throw new Error(`${text} not in the feed after 3 min`);
    await page.waitForTimeout(3000);
  }
}

const ann = await device();
await signUp(ann, `ann_${stamp}`);
await createChannel(ann, channel);
log('created a channel');
await createProject(ann, title, channel);
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
// web's requireViewer: a notice with a Sign in link (not a redirect).
await guest.locator('.auth-action-notice').getByText('Sign in to support this project.').waitFor();
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
// web's Search page: a tag finds the project as a result row.
for (let i = 0; ; i++) {
  await bob.goto(`${BASE}/search?q=seeds`);
  await bob.getByRole('heading', { name: 'Search', exact: true }).waitFor();
  const row = bob.locator('.result-card', { hasText: title });
  try {
    await row.waitFor({ timeout: 10_000 });
    break;
  } catch (e) {
    if (i >= 12) throw e;
  }
}
log('search by tag finds the project');

// The channel's page lists what is tagged with it.
for (let i = 0; ; i++) {
  await bob.goto(`${BASE}/channels/${channel}`);
  await bob.getByRole('heading', { name: channel }).waitFor({ timeout: 60_000 });
  const card = bob.locator('.project-card', { hasText: title });
  await bob.getByText(/Loading feed/).waitFor({ state: 'detached', timeout: 60_000 });
  if (await card.count()) break;
  if (i >= 20) throw new Error('the project never showed on its channel page');
  await bob.waitForTimeout(3000);
}
log('the channel page lists the project');

// web's tag chips: each tag has its scope icon and opens its page; on a phone the card shows
// one tag and "+1" opens web's ScopeSheet with both.
await guest.setViewportSize({ width: 390, height: 844 });
await guest.goto(`${BASE}/channels/${channel}`);
const tagged = guest.locator('.project-card', { hasText: title });
await tagged.waitFor({ timeout: 60_000 });
const firstTag = tagged.locator('a.tag-plain');
assert.equal(await firstTag.count(), 1, 'one tag shows on a phone');
assert.equal(await firstTag.getAttribute('href'), `/channels/${channel}`);
assert.equal(await firstTag.locator('svg').count(), 1, 'the tag has its channel icon');
await tagged.getByRole('button', { name: 'Show 1 more tags' }).click();
const sheet = guest.locator('.scope-sheet');
await sheet.locator('.scope-row', { hasText: 'testing' }).waitFor();
assert.equal(await sheet.locator('.scope-row').count(), 2, 'the sheet lists both tags');
await sheet.locator('.scope-row', { hasText: 'testing' }).click();
await guest.waitForURL(/\/channels\/testing/);
log('tags show their icons and open their channel; "+1" lists them all');

await ann.goto(`${BASE}/create/project`);
await ann.getByText('(0 left)').waitFor({ timeout: 60_000 });
log('a New member sees 0 projects left after a channel and a project');

assert.deepEqual(errors, [], 'no page errors');
log('all M2 project checks passed');
await browser.close();
