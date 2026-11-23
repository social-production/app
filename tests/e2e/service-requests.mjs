// Requests switched on by the plan (ADR 0059 stage A) end to end at phone width. Personal
// services can no longer be created, so a service's requests start from its operations plan:
// the two founders propose a bike co-op, file an operations plan that answers "Will this
// service take requests?" with a mode, approve it and move on to Activity. With no settings
// vote, Cleo (not a member) finds "Request service", sends a request and sees it Pending under
// "Your requests". collective-requests.mjs covers the vote that switches requests on later and
// a member planning a request into an activity.
//   node tests/e2e/service-requests.mjs http://localhost:4174 [screens dir]
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, addPlan, browser, continueSteps, device, errors, log, signUp, reviewPlan } from './lib.mjs';
import { advanceTo, signInFounders, until } from './event-flow.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const title = `Bike repairs ${stamp}`;
const planTitle = `Weekend repairs ${stamp}`;
const ask = `Fix my flat ${stamp}`;

const { ann, bob } = await signInFounders(phone);
await ann.goto(`${BASE}/create/project`);
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('What do you propose?').fill('We fix bikes on weekends.');
await continueSteps(ann, 3);
await ann.getByRole('button', { name: 'Publish proposal' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
const projectUrl = ann.url().split('?')[0];
await ann.getByRole('button', { name: /^Support/ }).click();
await bob.goto(projectUrl);
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await bob.getByRole('button', { name: /^Join / }).click();
await bob.getByRole('button', { name: '2 members' }).waitFor({ timeout: 60_000 });
await bob.getByRole('button', { name: /^Support/ }).click();
await until(ann, () => ann.getByRole('button', { name: '2 members' }), 'the second member');
await advanceTo(ann, bob, 'Plan');
log('the founders moved the proposal to its Plan phase');

await ann.reload();
await ann.getByRole('tab', { name: 'Participation' }).click();
await addPlan(ann, 'Operations plan');
const wizard = ann.getByRole('dialog', { name: 'Create Operations Plan' });
const nextPlan = () => wizard.getByRole('button', { name: 'Next' }).click();
await wizard.getByPlaceholder('Plan title').fill(planTitle);
await wizard.getByPlaceholder('Describe the plan').fill('Open the shed on weekends.');
await nextPlan(); // → plan type
await nextPlan(); // → location
await wizard.getByRole('textbox', { name: 'Place' }).fill('The shed');
await nextPlan(); // → requests
await wizard.getByText('Will this service take requests?').waitFor();
await wizard.getByLabel('Requests').selectOption({ index: 1 });
const mode = (await wizard.getByLabel('Requests').locator('option:checked').innerText()).trim();
await ann.screenshot({ path: `${out}/phone390-plan-requests.png` });
await nextPlan();
await wizard.getByPlaceholder('Explain how this plan responds to support').fill('Neighbours keep asking.');
await nextPlan();
await wizard.getByPlaceholder('Stage title').fill('Open the shed');
await wizard.getByPlaceholder('What happens in this stage?').fill('Unlock and set out tools.');
await nextPlan();
await wizard.getByRole('button', { name: 'Submit Operations Plan' }).click();
await ann.locator('.plan-card', { hasText: planTitle }).waitFor({ timeout: 60_000 });
log(`ann filed an operations plan that takes requests (${mode})`);

/** Rate every criterion "Support" and vote Yes; waits for `yes` votes on the card. */
async function assessPlan(page, yes) {
  await page.getByRole('tab', { name: 'Participation' }).click();
  await page.locator('.plan-card', { hasText: planTitle }).click();
  await reviewPlan(page);
  await page.locator('.plan-card', { hasText: `${yes} yes` }).waitFor({ timeout: 120_000 });
  await page.keyboard.press('Escape');
}
await assessPlan(ann, 1);
await until(bob, () => bob.locator('.plan-card', { hasText: planTitle }), 'the plan');
await assessPlan(bob, 2);
await until(bob, () => bob.locator('.plan-card.leading', { hasText: planTitle }), 'the plan to lead');
await advanceTo(ann, bob, 'Activity');
log('the plan made it a collective service as it reached Activity');

// No settings vote: the plan switched requests on.
const cleo = await device({ viewport: phone });
await signUp(cleo, `cleo${stamp}`);
const request = cleo.getByRole('button', { name: 'Request', exact: true });
for (let i = 0; i < 20; i++) {
  await cleo.goto(projectUrl);
  await cleo.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
  await cleo.getByRole('tab', { name: 'Participation' }).click();
  if (await request.waitFor({ timeout: 10_000 }).then(() => true, () => false)) break;
  if (i === 19) throw new Error('no "Request service" button: the plan did not switch requests on');
}
await request.click();
const composer = cleo.getByRole('dialog');
await composer.getByPlaceholder('Request title').fill(ask);
await composer.locator('textarea').first().fill('The back tyre keeps going flat.');
await composer.getByRole('button', { name: 'Request', exact: true }).click();
await cleo.getByRole('heading', { name: 'Your requests' }).waitFor({ timeout: 120_000 });
const mine = cleo.locator('.request-card-shell', { hasText: ask });
await mine.getByText('Pending', { exact: true }).waitFor({ timeout: 60_000 });
await cleo.screenshot({ path: `${out}/phone390-your-requests.png` });
log('cleo, not a member, sent a request with no settings vote; it reads Pending');

assert.deepEqual(errors, [], `page errors: ${errors.join('\n')}`);
await browser.close();
log('service requests ok');
