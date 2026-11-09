// Links between projects (ADR 0042) end to end against a local seed node started by
// scripts/dev-seed.sh (dev founders, so their votes count). Each founder creates a project;
// one proposes a link from the Links tab, both projects approve it (the second from the
// right rail's link vote), then they sever it (the second from the History tab's vote sheet).
// History lists both decisions, as web-backend's `build_link_decision_history_entries` does.
//   node tests/e2e/links.mjs http://localhost:4174
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn, rail, railShows, continueSteps } from './lib.mjs';

const [one, two] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const stamp = Date.now().toString(36).slice(-4);
const titles = [`Tool shed ${stamp}`, `Repair cafe ${stamp}`];
const shots = process.env.E2E_OUT ?? 'screens';
mkdirSync(shots, { recursive: true });

async function create(page, title) {
  await page.goto(`${BASE}/create/project`);
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByLabel('Title').fill(title);
  await page.getByLabel('Proposal description').fill('Lend and fix tools together.');
  await continueSteps(page, 3);
  await page.getByRole('button', { name: 'Publish project' }).click();
  await page.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
  return page.url();
}

/** Open the Links tab; reload until `locator` shows (the other project's side arrives by gossip). */
async function linksUntil(page, url, locator, what) {
  for (let i = 0; i < 40; i++) {
    await page.goto(url);
    await page.locator('.project-detail h1').waitFor({ state: 'attached', timeout: 60_000 });
    await page.getByRole('tab', { name: 'Links' }).click();
    await page
      .locator('.links-tab .network-stack')
      .first()
      .waitFor({ timeout: 180_000 })
      .catch(async (e) => {
        log(await page.locator('.links-tab').innerText().catch(() => 'no links tab'));
        throw e;
      });
    if (await locator().count()) return;
    await page.waitForTimeout(3000);
  }
  throw new Error(`timed out waiting for ${what}`);
}

/** Expand the card for `title` in the first `section` and press `button` on the viewer's own
 * side. A vote that decides the request moves the card out of the section; otherwise the
 * pressed button stays marked (votes are final). */
/** The History tab's card for `kind` with `state` (open, approved, rejected). */
async function historyCard(page, kind, state) {
  await page.getByRole('tab', { name: 'History' }).click();
  const card = page.locator(`.history-shell .history-card.${state}`, { hasText: kind }).first();
  await card.waitFor({ timeout: 180_000 }).catch(async (e) => {
    log(await page.locator('.history-shell').innerText().catch(() => 'no history tab'));
    throw e;
  });
  return card;
}

async function voteOn(page, section, title, button, decides = false) {
  const card = page.locator(section).first().locator('.link-card', { hasText: title });
  if ((await card.getAttribute('class'))?.includes('expanded') !== true) await card.locator('.toggle-button').click();
  await card.locator('.vote-button:not([disabled])', { hasText: button }).first().click();
  const done = decides
    ? card.waitFor({ state: 'detached', timeout: 120_000 })
    : card.locator('.vote-button.active', { hasText: button }).waitFor({ timeout: 120_000 });
  await done.catch(async (e) => {
    await page.screenshot({ path: `${shots}/links-vote-failed.png`, fullPage: true });
    log(await page.locator('.links-tab').innerText());
    throw e;
  });
}

const ann = await device({ viewport: { width: 390, height: 844 } });
await phraseSignIn(ann, one.phrase.split(' '));
const urlA = await create(ann, titles[0]);
const bob = await device();
bob.on('console', (m) => m.type() === 'error' && log('bob console:', m.text().slice(0, 300)));
await phraseSignIn(bob, two.phrase.split(' '));
const urlB = await create(bob, titles[1]);
log('each founder created a project');

// Founder one proposes the link; the search finds B through the node's index.
await linksUntil(ann, urlA, () => ann.getByRole('button', { name: 'Propose a new link' }), 'the Links tab');
await ann.screenshot({ path: `${shots}/links-empty-390.png` });
await ann.getByRole('button', { name: 'Propose a new link' }).click();
const composer = ann.getByRole('dialog', { name: 'Propose a new link' });
for (let i = 0; ; i++) {
  await composer.getByLabel('Search by name').fill(titles[1]);
  const hit = composer.locator('.search-result', { hasText: titles[1] });
  if (await hit.waitFor({ timeout: 5000 }).then(() => true, () => false)) {
    await hit.click();
    break;
  }
  if (i >= 30) throw new Error('the other project never showed in search');
  await composer.getByLabel('Search by name').fill('');
}
await composer.getByLabel('Why are these linked?').fill('The cafe borrows the shed tools.');
await composer.getByRole('button', { name: 'Create link vote' }).click();
await composer.waitFor({ state: 'hidden', timeout: 120_000 });
await ann.locator('.request-stack .link-card', { hasText: titles[1] }).waitFor({ timeout: 60_000 });
await voteOn(ann, '.request-stack', titles[1], 'Approve');
await ann.screenshot({ path: `${shots}/links-pending-390.png` });
log('founder one proposed the link and approved it for project A');

// web's rail: B's half waits on founder two under Votes to cast; Assess opens B's Links tab
// with the request expanded.
const linkRow = () => rail(bob).locator('.vote-row', { hasText: `Link vote · ${titles[0]}` });
await bob.goto(urlB);
await railShows(bob, linkRow, 'the link vote');
assert.match(await linkRow().innerText(), /Vote · link/i);
assert.match(await linkRow().innerText(), new RegExp(titles[1]));
await linkRow().getByRole('button', { name: 'Assess', exact: true }).click();
await bob.locator('.request-stack .link-card.expanded', { hasText: titles[0] }).waitFor({ timeout: 180_000 });
assert.match(bob.url(), /tab=links&linkRequest=/);
log('founder two opened the link vote from the rail');
await voteOn(bob, '.request-stack', titles[0], 'Approve', true);
await linksUntil(bob, urlB, () => bob.locator('.network-stack .link-card', { hasText: titles[0] }), 'the active link');
log('founder two approved it for project B: the link is active');

await linksUntil(ann, urlA, () => ann.locator('.network-stack .link-card', { hasText: titles[1] }), 'the link in A');
await ann.screenshot({ path: `${shots}/links-active-390.png` });
// A string `hasText` ignores case, and "Sever link decision" contains "link decision".
await historyCard(ann, /^Link decision/, 'approved');
log("A's History lists the approved link decision");
await ann.getByRole('tab', { name: 'Links' }).click();
await ann.locator('.network-stack').first().locator('.link-card .toggle-button').click();
await ann.getByRole('button', { name: 'Start sever vote' }).click();
await voteOn(ann, '.network-stack', titles[1], 'Remove link');
await linksUntil(bob, urlB, () => bob.locator('.network-stack .link-card .side-vote-line'), 'the sever vote in B');
// Founder two votes on B's half from History (web's DecisionHistoryCard opens the vote sheet).
await (await historyCard(bob, 'Sever link decision', 'open')).locator('.history-toggle').click();
const sheet = bob.getByRole('dialog', { name: 'Sever' });
await sheet.getByText(titles[0]).first().waitFor({ timeout: 60_000 });
await sheet.getByRole('button', { name: 'Yes', exact: true }).click();
await sheet.waitFor({ state: 'hidden', timeout: 60_000 });
// Stay on the page until the vote decides it (leaving at once can drop the write).
await bob
  .locator('.history-shell .history-card.approved', { hasText: 'Sever link decision' })
  .waitFor({ timeout: 180_000 });
await bob.screenshot({ path: `${shots}/links-history-390.png`, fullPage: true });
log('founder two voted on the sever from History');
await linksUntil(bob, urlB, () => bob.getByText('No active links yet.'), 'the link to end');
await bob.getByRole('button', { name: /^Show past link votes/ }).click();
log('both projects voted to sever: the link is historical');
await ann.goto(urlA);
await historyCard(ann, 'Sever link decision', 'approved');
log("A's History lists the approved sever decision");

assert.deepEqual(errors, [], 'no page errors');
log('links ok');
await browser.close();
