// Software projects (ADR 0086) end to end at phone width, against a local seed node with the
// dev founders (scripts/dev-seed.sh). Ann creates a project; both founders move it to
// Production Plan and approve Ann's software plan (with its repository), then move it to
// Activity, where web's software panel shows the repository and Ann as its merger. Ann submits
// a pull request through web's wizard; Ann approves it in the panel, Bob from his rail
// ("Vote · pull request" → Assess opens the wizard). Ann's rail then says "Action · merge
// needed" and her notifications "Pull request approved", which opens Record merge; both
// confirm it, and it leaves "In-progress pull requests". Bob's Repository ▾ offers web's two
// votes; he asks to be granted merge capability, Ann votes on it from her rail (Take part's
// vote sheet), and the History tab and Past activity list the software decisions (phase c).
//   node tests/e2e/software.mjs http://localhost:4174 [screens dir]
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, addPlan, browser, continueSteps, errors, log, rail, railShows } from './lib.mjs';
import { advanceTo, signInFounders, until } from './event-flow.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const stamp = Date.now().toString(36).slice(-5);
const title = `Watering app ${stamp}`;
const planTitle = `Timer build ${stamp}`;
const repo = `https://example.org/garden-${stamp}.git`;
const prTitle = `Morning timer ${stamp}`;

const { ann, bob } = await signInFounders({ width: 390, height: 844 });

await ann.goto(`${BASE}/create/project`);
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('Proposal description').fill('An app that waters the beds.');
await continueSteps(ann, 3);
await ann.getByRole('button', { name: 'Publish proposal' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
const projectUrl = ann.url().split('?')[0];
await ann.getByRole('button', { name: /^Support/ }).click();
await bob.goto(projectUrl);
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await bob.getByRole('button', { name: 'Join project' }).click();
await bob.getByRole('button', { name: '2 members' }).waitFor({ timeout: 60_000 });
await bob.getByRole('button', { name: /^Support/ }).click();
await until(ann, () => ann.getByRole('button', { name: '2 members' }), 'the second member');
await advanceTo(ann, bob, 'Production Plan');
log('the founders moved the project to Production Plan');

// Ann's production plan picks web's Software type and names the official repository.
await ann.reload();
await ann.getByRole('tab', { name: 'Participation' }).click();
await addPlan(ann, 'Production plan');
const wizard = ann.getByRole('dialog', { name: 'Create Production Plan' });
const next = () => wizard.getByRole('button', { name: 'Next' }).click();
await wizard.getByPlaceholder('Plan title').fill(planTitle);
await wizard.getByPlaceholder('Describe the plan').fill('Write the timer and the sensors.');
await next();
await wizard.getByLabel('Plan type').selectOption('software');
await next();
await wizard.getByLabel('Repository URL').fill(repo);
await next();
await wizard.getByPlaceholder('Explain how this plan responds to support').fill('Both of us asked.');
await next();
await wizard.getByPlaceholder('Stage title').fill('Timer');
await wizard.getByPlaceholder('What happens in this stage?').fill('Write the timer.');
await next();
await wizard.getByRole('button', { name: 'Submit Production Plan' }).click();
await ann.locator('.plan-card', { hasText: planTitle }).waitFor({ timeout: 60_000 });

/** Rate every criterion "Support" and vote Yes; waits for `yes` votes on the card. */
async function assessPlan(page, yes) {
  await page.getByRole('tab', { name: 'Participation' }).click();
  await page.locator('.plan-card', { hasText: planTitle }).click();
  const sheet = page.locator('.plan-sheet');
  await sheet.locator('.criterion-row').first().waitFor();
  for (const row of await sheet.locator('.criterion-row').all()) {
    await row.getByRole('button', { name: /^4(\s*Support)?$/ }).click();
    await row.getByText('Your rating: Support').waitFor({ timeout: 30_000 });
  }
  await sheet.getByRole('button', { name: 'Yes', exact: true }).click();
  await sheet.getByText('Your vote is final once cast.').waitFor({ timeout: 30_000 });
  await page.locator('.plan-card', { hasText: `${yes} yes` }).waitFor({ timeout: 120_000 });
  await page.keyboard.press('Escape');
}
await assessPlan(ann, 1);
await until(bob, () => bob.locator('.plan-card', { hasText: planTitle }), 'the plan');
await assessPlan(bob, 2);
await until(bob, () => bob.locator('.plan-card.leading', { hasText: planTitle }), 'the plan to lead');
await advanceTo(ann, bob, 'Activity');
log('the software plan leads and the project reached Activity');

/** The Participation tab, reloaded until the panel shows `locator`. */
async function panel(page, locator, what) {
  for (let i = 0; i < 40; i++) {
    await page.goto(projectUrl);
    await page.locator('.project-detail h1').waitFor({ state: 'attached', timeout: 60_000 });
    await page.getByRole('tab', { name: 'Participation' }).click();
    if (await locator().waitFor({ timeout: 5_000 }).then(() => true, () => false)) return;
    await page.waitForTimeout(3000);
  }
  throw new Error(`timed out waiting for ${what}`);
}

const repoCell = (page) => page.locator('.software-panel .repo-cell', { hasText: 'Repository' });
await panel(ann, () => repoCell(ann).getByText(repo.replace('https://', '')), 'the repository');
const mergers = await ann.locator('.software-panel .repo-cell', { hasText: 'Mergers' }).innerText();
assert.doesNotMatch(mergers, /No one yet/, 'the plan author may merge');
await ann.screenshot({ path: `${out}/software-panel-390.png` });
log('the panel shows the repository and its merger');

await ann.getByRole('button', { name: 'Pull request' }).click();
const submit = ann.getByRole('dialog', { name: 'Submit pull request' });
const step = () => submit.getByRole('button', { name: 'Next' }).click();
await submit.getByPlaceholder('Pull request title').fill(prTitle);
await step();
await submit.getByPlaceholder('PR number or ID').fill('#12');
await submit.getByPlaceholder('Pull request URL').fill('https://example.org/garden/pull/12');
await step();
await submit.getByPlaceholder('What does this pull request change?').fill('Waters at six.');
await step();
await submit.getByRole('button', { name: 'Submit for approval' }).click();
await submit.waitFor({ state: 'hidden', timeout: 60_000 });
log('ann submitted a pull request');

/** Open the queue and the pull request's card, then its action (`Assess` or `Record merge`). */
async function openAction(page, action, what) {
  const card = () => page.locator('.pr-card', { hasText: prTitle });
  const button = () => card().getByRole('button', { name: action, exact: true });
  for (let i = 0; i < 40; i++) {
    await panel(page, () => page.getByRole('button', { name: /In-progress pull requests/ }), what);
    await page.getByRole('button', { name: /In-progress pull requests/ }).click();
    await card().locator('.pr-card-body-shell').click();
    if (await button().waitFor({ timeout: 5_000 }).then(() => true, () => false)) {
      await button().click();
      return;
    }
    await page.waitForTimeout(3000);
  }
  throw new Error(`timed out waiting for ${what}`);
}

async function voteYes(page, dialog, what) {
  await openAction(page, 'Assess', what);
  const sheet = page.getByRole('dialog', { name: dialog });
  await sheet.locator('.choice-bar .choice.yes').click();
  await sheet.waitFor({ state: 'hidden', timeout: 60_000 });
}

await voteYes(ann, 'Pull request vote needed', 'the pull request');
// Bob's rail: web's "Vote · pull request"; Assess opens the pull request's wizard.
await bob.goto(projectUrl);
const prVote = () => rail(bob).locator('.vote-row', { hasText: `Pull request vote · ${prTitle}` });
await railShows(bob, prVote, 'the pull request vote', 30_000);
assert.match(await prVote().innerText(), /Vote · pull request/i);
await bob.screenshot({ path: `${out}/software-rail-vote-390.png` });
await prVote().getByRole('button', { name: 'Assess' }).click();
const bobVote = bob.getByRole('dialog', { name: 'Pull request vote needed' });
await bobVote.locator('.choice-bar .choice.yes').click({ timeout: 120_000 });
await bobVote.waitFor({ state: 'hidden', timeout: 60_000 });
log('both approved the pull request, bob from his rail');

// Ann may merge: her rail says a merge is needed; web's pr-approved opens Record merge.
await ann.goto(projectUrl);
const needed = () => rail(ann).locator('.vote-row', { hasText: `Merge needed · ${prTitle}` });
await railShows(ann, needed, 'the merge needed action', 30_000);
assert.match(await needed().innerText(), /Action · merge needed/i);
await needed().getByRole('button', { name: 'Record merge', exact: true }).waitFor();
const approved = () => ann.locator('.notification-card', { hasText: 'Pull request approved' });
for (let i = 0; i < 40 && !(await approved().count()); i++) {
  await ann.goto(`${BASE}/notifications`);
  await approved().first().waitFor({ timeout: 10_000 }).catch(() => {});
}
assert.match(await approved().first().innerText(), /your pull request is approved for merge/);
await approved().first().click();
const merge = ann.getByRole('dialog', { name: 'Merge needed' });
await merge.getByLabel('Merge ID').fill('a1b2c3');
await merge.getByLabel('Merge link').fill('https://example.org/garden/commit/a1b2c3');
await merge.getByRole('button', { name: 'Submit merge details' }).click();
await merge.waitFor({ state: 'hidden', timeout: 60_000 });
log('ann recorded the merge');

await voteYes(bob, 'Merge confirmation needed', 'the merge confirmation');
await voteYes(ann, 'Merge confirmation needed', 'the merge confirmation on ann');
const queue = () => ann.locator('.pr-card', { hasText: prTitle });
for (let i = 0; i < 40 && (await queue().count()); i++) {
  await ann.waitForTimeout(3000);
  await panel(ann, () => repoCell(ann), 'the panel');
}
assert.equal(await queue().count(), 0, 'a confirmed merge leaves the in-progress list');
log('both confirmed the merge; the pull request left the in-progress list');

await panel(bob, () => bob.getByRole('button', { name: 'Repository', exact: true }), 'Repository ▾');
await bob.getByRole('button', { name: 'Repository', exact: true }).click();
const menu = bob.getByRole('dialog', { name: 'Repository' });
await menu.getByRole('button', { name: /Replace repository/ }).waitFor();
await menu.getByRole('button', { name: /Grant or revoke merge capability/ }).waitFor();
log("bob's Repository ▾ offers web's two votes");

// Bob asks for merge capability; Ann votes from her rail in Take part's vote sheet.
await menu.getByRole('button', { name: /Grant or revoke merge capability/ }).click();
const ask = bob.getByRole('dialog', { name: 'Merge capability change' });
await ask.getByRole('button', { name: 'Next' }).click();
await ask.getByRole('button', { name: 'Next' }).click();
await ask.getByRole('button', { name: 'Submit vote request' }).click();
await ask.waitFor({ state: 'hidden', timeout: 60_000 });
const grant = () => rail(ann).locator('.vote-row', { hasText: 'Merge capability · Grant' });
await ann.goto(projectUrl);
await railShows(ann, grant, 'the merge capability vote', 60_000);
assert.match(await grant().innerText(), /Vote · merge capability/i);
await grant().getByRole('button', { name: 'Assess' }).click();
const sheet = ann.getByRole('dialog', { name: 'Merge capability' });
await sheet.getByText('Grant merge capability').waitFor({ timeout: 120_000 });
await sheet.locator('.choice-bar .choice.yes').click();
await sheet.waitFor({ state: 'hidden', timeout: 60_000 });

// The History tab and Past activity list the software decisions. Stay on the page: leaving
// it at once would drop the vote still being written.
await ann.getByRole('tab', { name: 'History' }).click();
const card = (kind) => ann.locator('.history-shell .history-card', { hasText: kind }).first();
await card('Merge capability change').filter({ hasText: '1 yes' }).waitFor({ timeout: 120_000 });
await card('Merge confirmation').waitFor({ timeout: 60_000 });
log('bob asked for merge capability and ann voted on it from her rail');
await ann.screenshot({ path: `${out}/software-history-390.png` });
await ann.getByRole('tab', { name: 'Participation' }).click();
await ann.getByText('Past activity').click();
await ann.locator('.unified-history .history-card', { hasText: 'Merge confirmation' }).waitFor();
log('History and Past activity list the software decisions');

assert.deepEqual(errors, [], `page errors: ${errors.join('\n')}`);
await browser.close();
log('software: passed');
