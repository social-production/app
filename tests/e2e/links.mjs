// Links between projects (ADR 0042) end to end against a local seed node started by
// scripts/dev-seed.sh (dev founders, so their votes count). Each founder creates a project;
// one proposes a link from the Links tab, both projects approve it, then they sever it.
//   node tests/e2e/links.mjs http://localhost:4174
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn } from './lib.mjs';

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
  for (let i = 0; i < 3; i++) await page.getByRole('button', { name: 'Continue' }).click();
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
    await page.locator('.links-tab .network-stack').first().waitFor({ timeout: 120_000 });
    if (await locator().count()) return;
    await page.waitForTimeout(3000);
  }
  throw new Error(`timed out waiting for ${what}`);
}

/** Expand the first card in `section` and press `button` on this project's (enabled) side. */
async function voteOn(page, section, button) {
  const card = page.locator(`${section} .link-card`).first();
  if ((await card.getAttribute('class'))?.includes('expanded') !== true) await card.locator('.toggle-button').click();
  await card.locator('.vote-button:not([disabled])', { hasText: button }).first().click();
  await card.locator('.vote-button.active', { hasText: button }).waitFor({ timeout: 120_000 });
}

const ann = await device({ viewport: { width: 390, height: 844 } });
await phraseSignIn(ann, one.phrase.split(' '));
const urlA = await create(ann, titles[0]);
const bob = await device();
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
await voteOn(ann, '.request-stack', 'Approve');
await ann.screenshot({ path: `${shots}/links-pending-390.png` });
log('founder one proposed the link and approved it for project A');

await linksUntil(bob, urlB, () => bob.locator('.request-stack .link-card', { hasText: titles[0] }), 'the request in B');
await voteOn(bob, '.request-stack', 'Approve');
await linksUntil(bob, urlB, () => bob.locator('.network-stack .link-card', { hasText: titles[0] }), 'the active link');
log('founder two approved it for project B: the link is active');

await linksUntil(ann, urlA, () => ann.locator('.network-stack .link-card', { hasText: titles[1] }), 'the link in A');
await ann.screenshot({ path: `${shots}/links-active-390.png` });
await ann.locator('.network-stack .link-card .toggle-button').first().click();
await ann.getByRole('button', { name: 'Start sever vote' }).click();
await voteOn(ann, '.network-stack', 'Remove link');
await linksUntil(bob, urlB, () => bob.locator('.network-stack .vote-block'), 'the sever vote in B');
await voteOn(bob, '.network-stack', 'Remove link');
await linksUntil(bob, urlB, () => bob.getByText('No active links yet.'), 'the link to end');
await bob.getByRole('button', { name: /^Show past link votes/ }).click();
log('both projects voted to sever: the link is historical');

assert.deepEqual(errors, [], 'no page errors');
log('links ok');
await browser.close();
