// M3a governance end to end against a local seed node started by scripts/dev-seed.sh, whose
// dev hApp makes the two identities in fixtures/dev-founders.json founders (Full standing).
// One proposes an edit, an update and a phase move; the other votes them through.
//   node tests/e2e/governance.mjs http://localhost:4174
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn } from './lib.mjs';

const [one, two] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const stamp = Date.now().toString(36).slice(-4);
const title = `Bike kitchen ${stamp}`;

const ann = await device();
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

async function voteYes(page) {
  await until(page, () => page.getByRole('button', { name: 'Vote', exact: true }), 'an open proposal');
  await page.getByRole('button', { name: 'Vote', exact: true }).click();
  await page.locator('.choice-bar .choice.yes').click();
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

await ann.reload();
await ann.getByRole('tab', { name: 'Participation' }).click();
await ann.getByRole('button', { name: /Propose moving to Production Plan/ }).click();
await voteYes(bob);
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
  for (const row of await sheet.locator('.criterion-row').all()) {
    await row.getByRole('button', { name: /^4\s*Support$/ }).click();
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
await ann.getByRole('button', { name: /^Propose moving to / }).click();
await voteYes(bob);
const pastPlan = () => bob.locator('.feed-card-stage').filter({ hasNotText: 'Production Plan' });
await until(bob, pastPlan, 'the move past the plan');
log('the approved plan let the project leave Production Plan');

assert.deepEqual(errors, [], 'no page errors');
log('all M3a + M3b governance checks passed');
await browser.close();
