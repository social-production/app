// Collective service requests (ADR 0089) end to end at phone width. The two founders take a
// collective service to its Activity phase (operations plan, votes), then vote requests on in
// Settings ▾ → Request settings (reason, Start vote, a Yes from Take part). Cleo, who isn't a
// member, sends a request; Bob (any member) opens Requests → Plan request, which opens the
// activity wizard with the request filled in, and creates the activity with its role. Cleo's
// request reads Accepted; the planned activity waits for its role, and Ann takes it from her
// right rail's Project Activity row.
//   node tests/e2e/collective-requests.mjs http://localhost:4174 [screens dir]
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, addPlan, browser, continueSteps, device, errors, log, rail, railShows, signUp } from './lib.mjs';
import { advanceTo, signInFounders, until, voteYes } from './event-flow.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const title = `Bike co-op ${stamp}`;
const planTitle = `Thursday evenings ${stamp}`;
const ask = `Fix my flat ${stamp}`;

const { ann, bob } = await signInFounders(phone);
await ann.goto(`${BASE}/create/project`);
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('Proposal description').fill('We fix bikes together.');
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
await advanceTo(ann, bob, 'Operations Plan');
log('the founders moved the collective service to Operations Plan');

await ann.reload();
await ann.getByRole('tab', { name: 'Participation' }).click();
await addPlan(ann, 'Operations plan');
const wizard = ann.getByRole('dialog', { name: 'Create Operations Plan' });
const nextPlan = () => wizard.getByRole('button', { name: 'Next' }).click();
await wizard.getByPlaceholder('Plan title').fill(planTitle);
await wizard.getByPlaceholder('Describe the plan').fill('Open the shed on Thursday evenings.');
await nextPlan(); // → plan type
await nextPlan(); // → location
await wizard.getByRole('textbox', { name: 'Place' }).fill('The shed');
await nextPlan();
await wizard.getByPlaceholder('Explain how this plan responds to support').fill('Both of us asked.');
await nextPlan();
await wizard.getByPlaceholder('Stage title').fill('Open the shed');
await wizard.getByPlaceholder('What happens in this stage?').fill('Unlock and set out tools.');
await nextPlan();
await wizard.getByRole('button', { name: 'Submit Operations Plan' }).click();
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
log('the operations plan leads and the service reached Activity');

/** The Participation tab, reloaded until `name`'s toolbar button shows; then press it. */
async function press(page, name) {
  for (let i = 0; i < 20; i++) {
    await page.goto(projectUrl);
    await page.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
    await page.getByRole('tab', { name: 'Participation' }).click();
    const button = page.getByRole('button', { name, exact: true });
    if (await button.waitFor({ timeout: 10_000 }).then(() => true, () => false)) {
      await button.click();
      return;
    }
  }
  throw new Error(`no "${name}" button`);
}

// Requests start off (web's collective default); the members vote them on with a reason.
await press(ann, 'Settings ▾');
await ann.getByRole('dialog', { name: 'Settings' }).getByRole('button', { name: /Request settings/ }).click();
const settings = ann.getByRole('dialog', { name: 'Request settings' });
await settings.getByLabel('Enable requests').check();
await settings.getByPlaceholder(/Explain why the request flow should change/).fill('People keep asking.');
await settings.getByRole('button', { name: 'Start vote' }).click();
await settings.waitFor({ state: 'hidden', timeout: 60_000 });
await bob.goto(projectUrl);
await voteYes(bob);
await bob.locator('.vote-sheet').getByText('Calendar and direct').first().waitFor({ timeout: 30_000 }).catch(() => {});
log('ann started the request settings vote and bob voted yes from Take part');

const cleo = await device({ viewport: phone });
await signUp(cleo, `cleo${stamp}`);
await press(cleo, 'Request service');
const composer = cleo.getByRole('dialog');
await composer.getByPlaceholder('Request title').fill(ask);
await composer.locator('textarea').first().fill('The back tyre keeps going flat.');
await composer.getByRole('button', { name: 'Request', exact: true }).click();
await cleo.getByRole('heading', { name: 'Your requests' }).waitFor({ timeout: 120_000 });
await cleo.locator('.request-card-shell', { hasText: ask }).waitFor({ timeout: 60_000 });
log('cleo, not a member, sent a request');

// Bob (any member) plans it: web's Plan request opens the activity wizard with it filled in.
await press(bob, 'Requests');
const card = bob.locator('.request-card-shell', { hasText: ask });
// The request's group chat reaches the members by gossip; a new chat can take minutes.
const asked = Date.now();
for (let i = 0; i < 60 && !(await card.count()); i++) {
  await press(bob, 'Requests');
  await card.waitFor({ timeout: 10_000 }).catch(() => {});
}
log(`the request reached bob after ${Math.round((Date.now() - asked) / 1000)} s`);
if (!(await card.count())) {
  await bob.screenshot({ path: `${out}/collective-requests-missing.png`, fullPage: true });
  log('bob sees:', await bob.locator('.overlay-root').last().innerText().catch(() => 'no sheet'));
  await bob.goto(`${BASE}/messages`);
  await bob.waitForTimeout(8000);
  log('bob messages:', await bob.locator('main').innerText().catch(() => 'no messages'));
}
await card.locator('summary').click();
await card.getByRole('button', { name: 'Plan request' }).click();
await bob.screenshot({ path: `${out}/phone390-collective-plan.png` });
assert.equal(await bob.getByPlaceholder('Activity title').inputValue(), ask);
const next = () => bob.getByRole('button', { name: 'Next', exact: true }).click();
await next(); // → schedule (web's default time)
await next(); // → place
await bob.getByPlaceholder('Search a place').fill('The shed');
await next(); // → plan stage
await bob.locator('select:visible').first().selectOption({ index: 1 });
await next(); // → roles: web's "Service lead"
assert.equal(await bob.getByPlaceholder('For example: Intake desk').first().inputValue(), 'Service lead');
await next(); // → note: the request's details
await next();
await bob.getByRole('button', { name: 'Create activity' }).click();
const row = (page) => page.locator('.agenda-row', { hasText: ask });
await row(bob).waitFor({ timeout: 60_000 });
assert.match(await row(bob).locator('.agenda-chip').innerText(), /1 more needed/);
log('bob planned the request into an activity that waits for its role');

for (let i = 0; i < 20; i++) {
  await press(cleo, 'Your requests');
  const mine = cleo.locator('.request-card-shell', { hasText: ask });
  if (await mine.getByText('Accepted', { exact: true }).waitFor({ timeout: 10_000 }).then(() => true, () => false)) break;
  if (i === 19) throw new Error("cleo's request never read Accepted");
}
await cleo.screenshot({ path: `${out}/phone390-collective-accepted.png` });
log("cleo's request reads Accepted");

// Ann signs up from her right rail: the planned activity is an ordinary activity.
await ann.goto(projectUrl);
const activityRow = () => rail(ann).locator('.activity-row', { hasText: ask });
await railShows(ann, activityRow, 'the planned activity', 30_000);
await activityRow().locator('.activity-open-button').click();
const sheet = ann.getByRole('dialog', { name: ask });
await sheet.getByRole('button', { name: 'Take role' }).click();
await sheet.getByRole('button', { name: 'Leave role' }).waitFor({ timeout: 60_000 });
log('ann took the Service lead role from her rail');

assert.deepEqual(errors, [], `page errors: ${errors.join('\n')}`);
await browser.close();
log('collective-requests: ok');
