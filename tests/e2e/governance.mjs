// M3a governance end to end against a local seed node started by scripts/dev-seed.sh, whose
// dev hApp makes the two identities in fixtures/dev-founders.json founders (Full standing).
// One proposes an edit, an update and a phase move; the other votes them through.
//   node tests/e2e/governance.mjs http://localhost:4174
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn } from './lib.mjs';

const [one, two] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const stamp = Date.now().toString(36).slice(-4);
const title = `Bike kitchen ${stamp}`;

// Founder one is a phone (390px, web's first check); screenshots of the Take part tab go to E2E_OUT.
const ann = await device({ viewport: { width: 390, height: 844 } });
const shots = process.env.E2E_OUT ?? 'screens';
mkdirSync(shots, { recursive: true });
await phraseSignIn(ann, one.phrase.split(' '));
await ann.goto(`${BASE}/create/project`);
await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('Proposal description').fill('Fix bikes together on Saturdays.');
for (let i = 0; i < 3; i++) await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByRole('button', { name: 'Publish project' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
const projectUrl = ann.url();
await ann.getByRole('button', { name: /^Support/ }).click();
log('founder one created the project and supports it');

const bob = await device();
await phraseSignIn(bob, two.phrase.split(' '));
await bob.goto(projectUrl);
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await bob.getByRole('button', { name: 'Join project' }).click();
await bob.getByRole('button', { name: '2 members' }).waitFor({ timeout: 60_000 });
await bob.getByRole('button', { name: /^Support/ }).click();
log('founder two joined and supports it');

/** Reload until the loaded project page shows `locator` (gossip between the two agents). */
async function until(page, locator, what) {
  for (let i = 0; i < 40; i++) {
    // Attached, not visible: the title sits in the Details tab, which may be hidden.
    await page.locator('.project-detail h1').waitFor({ state: 'attached', timeout: 60_000 });
    if (await locator().count()) return;
    await page.waitForTimeout(3000);
    await page.reload();
  }
  throw new Error(`timed out waiting for ${what}`);
}

/** Vote Yes on the open proposal. Edits and updates are voted on in the Details tab's dock,
 * phase moves in the Take part tab's (as on web). */
async function voteYes(page, tab) {
  const vote = () => page.getByRole('button', { name: 'Vote', exact: true });
  await page.keyboard.press('Escape'); // a plan sheet left open would cover the tabs
  for (let i = 0; ; i++) {
    await page.locator('.project-detail h1').waitFor({ state: 'attached', timeout: 60_000 });
    if (tab) await page.getByRole('tab', { name: tab }).click();
    if (await vote().count()) break;
    if (i >= 40) throw new Error('timed out waiting for an open proposal');
    await page.waitForTimeout(3000);
    await page.reload();
  }
  await vote().click();
  await page.locator('.choice-bar .choice.yes').click();
}

/** web's Phase ▾ menu: choose "Advance to …", then say why in its sheet. */
async function advance(page) {
  await page.getByRole('button', { name: 'Phase', exact: true }).click();
  await page.getByRole('dialog', { name: 'Change phase' }).getByRole('button', { name: /^Advance to / }).click();
  const sheet = page.getByRole('dialog', { name: /^Advance to / });
  await sheet.getByPlaceholder('State why this phase change should happen now.').fill('We are ready.');
  await sheet.getByRole('button', { name: /^Advance to / }).click();
  await sheet.waitFor({ state: 'hidden', timeout: 60_000 });
}

await ann.reload();
await until(ann, () => ann.getByRole('button', { name: '2 members' }), 'the second member');
await ann.getByRole('button', { name: 'Edit', exact: true }).click();
await ann.locator('.edit-sheet').getByLabel('Title').fill(`${title} co-op`);
await ann.getByRole('button', { name: 'Propose edit' }).click();
await voteYes(bob);
await until(bob, () => bob.getByRole('heading', { name: `${title} co-op` }), 'the edit to apply');
log('an edit passed with both founders and applied');

await ann.reload();
await ann.getByRole('button', { name: 'Add update' }).click();
await ann.locator('.update-sheet textarea').fill('First repair night went well.');
await ann.getByRole('button', { name: 'Post update' }).click();
await voteYes(bob);
await until(bob, () => bob.locator('.update-card', { hasText: 'First repair night' }), 'the update');
log('an update passed and is listed');
// web's feed card shows the newest update ("Latest: …") once the index refreshes.
await bob.goto(`${BASE}/search?q=${encodeURIComponent(title)}`);
const latest = bob.locator('.project-card', { hasText: title }).getByText('Latest: First repair night went well.');
for (let i = 0; i < 20 && !(await latest.count()); i++) {
  await bob.waitForTimeout(3000);
  await bob.reload();
}
await latest.waitFor();
await bob.goto(projectUrl);
log('the feed card shows the update as Latest');

await ann.reload();
await ann.getByRole('tab', { name: 'Participation' }).click();
await ann.screenshot({ path: `${shots}/take-part-proposal-390.png` });
assert.deepEqual(await ann.locator('[role=tab][aria-selected=true]').allInnerTexts(), ['Take part']);

// M3d: web's "Add value"; both founders rate it 8/10, so plans are judged against it.
const valueLabel = `Quiet at night ${stamp}`;
await ann.getByRole('button', { name: 'Add value' }).click();
const valueSheet = ann.getByRole('dialog', { name: 'Add value' });
await valueSheet.getByPlaceholder(/^Add a value/).fill(valueLabel);
await valueSheet.getByRole('button', { name: 'Add value' }).click();
await valueSheet.waitFor({ state: 'hidden', timeout: 60_000 });
/** Rate the value 8/10 and wait until the reload carries the vote. */
async function rateValue(page) {
  const card = page.locator('.value-card', { hasText: valueLabel });
  await card.getByRole('button', { name: 'Importance 8 of 10' }).click();
  await card.getByText('Your vote: 8/10').waitFor({ timeout: 60_000 });
}
await rateValue(ann);
await until(bob, () => bob.locator('.value-card', { hasText: valueLabel }), 'the value');
await bob.getByRole('tab', { name: 'Participation' }).click();
await rateValue(bob);
await bob.locator('.value-card', { hasText: 'Community average: 8/10' }).waitFor({ timeout: 120_000 });
await ann.screenshot({ path: `${shots}/take-part-values-390.png` });
log('a value was added and both founders rated it');

await advance(ann);
await voteYes(bob, 'Participation');
await until(bob, () => bob.locator('.feed-card-stage', { hasText: 'Production Plan' }), 'the phase move');
log('the project moved to Production Plan');

// M3b: a production plan, rated and approved by both, lets the project leave Production Plan.
const planTitle = `Saturday workshop ${stamp}`;
await ann.reload();
await ann.getByRole('tab', { name: 'Participation' }).click();
await ann.getByRole('button', { name: 'Add plan' }).click();
const wizard = ann.getByRole('dialog', { name: 'Create Production Plan' });
await wizard.getByPlaceholder('Plan title').fill(planTitle);
await wizard.getByPlaceholder('Describe the plan').fill('Open the workshop every Saturday morning.');
await wizard.getByRole('button', { name: 'Next' }).click(); // overview → plan type
await wizard.getByRole('button', { name: 'Next' }).click(); // → location
await wizard.getByLabel('Location').fill('Community hall');
await wizard.getByRole('button', { name: 'Next' }).click();
await wizard.getByPlaceholder('Explain how this plan responds to support').fill('Both supporters can come.');
await wizard.getByRole('button', { name: 'Next' }).click();
await wizard.getByPlaceholder('Stage title').fill('Set up benches');
await wizard.getByPlaceholder('What happens in this stage?').fill('Borrow tools and benches.');
await wizard.getByRole('button', { name: 'Next' }).click(); // → review
await wizard.getByRole('button', { name: 'Submit Production Plan' }).click();
await ann.locator('.plan-card', { hasText: planTitle }).waitFor({ timeout: 60_000 });
log('founder one filed a production plan');

/** Open the plan, rate every criterion "Support", vote Yes, and wait until the tally shows
 * `yes` votes (writes run one at a time; leaving the page earlier would cancel them). */
async function assess(page, yes) {
  await page.getByRole('tab', { name: 'Participation' }).click();
  await page.locator('.plan-card', { hasText: planTitle }).click();
  const sheet = page.locator('.plan-sheet');
  await sheet.locator('.criterion-row', { hasText: valueLabel }).getByText('Shared value').waitFor();
  for (const row of await sheet.locator('.criterion-row').all()) {
    await row.getByRole('button', { name: /^4(\s*Support)?$/ }).click();
    await row.getByText('Your rating: Support').waitFor({ timeout: 30_000 });
  }
  await sheet.getByRole('button', { name: 'Yes', exact: true }).click();
  await sheet.getByText('Your vote is final once cast.').waitFor({ timeout: 30_000 });
  await page.locator('.plan-card', { hasText: `${yes} yes` }).waitFor({ timeout: 120_000 });
}

await assess(ann, 1);
await until(bob, () => bob.locator('.plan-card', { hasText: planTitle }), 'the plan');
await assess(bob, 2);
await until(bob, () => bob.locator('.plan-card.leading', { hasText: planTitle }), 'the plan to lead');
log('both rated and approved the plan; it leads');

await ann.reload();
await ann.getByRole('tab', { name: 'Participation' }).click();
await ann.screenshot({ path: `${shots}/take-part-plan-390.png` });
await advance(ann);
await voteYes(bob, 'Participation');
const pastPlan = () => bob.locator('.feed-card-stage').filter({ hasNotText: 'Production Plan' });
await until(bob, pastPlan, 'the move past the plan');
log('the approved plan let the project leave Production Plan');

// M3c: web's Phase menu → Return to an earlier phase; the other founder's vote sends it back.
await ann.reload();
await ann.getByRole('tab', { name: 'Participation' }).click();
await ann.getByRole('button', { name: 'Phase', exact: true }).click();
await ann.getByRole('dialog', { name: 'Change phase' }).getByRole('button', { name: /^Return to an earlier phase/ }).click();
const returning = ann.getByRole('dialog', { name: /^Return to / });
await returning.getByPlaceholder(/return to an earlier planning phase/).fill('The plan needs another look.');
await returning.getByRole('button', { name: 'Return', exact: true }).click();
await returning.waitFor({ state: 'hidden', timeout: 60_000 });
// Proposing a return is not a Yes (as on web), so both founders vote.
await voteYes(bob, 'Participation');
await voteYes(ann, 'Participation');
const backToPlan = () => bob.locator('.feed-card-stage', { hasText: 'Production Plan' });
await until(bob, backToPlan, 'the return to Production Plan');
log('a return to Production Plan passed');

assert.deepEqual(errors, [], 'no page errors');
log('all M3a + M3b + M3c + M3d governance checks passed');
await browser.close();
