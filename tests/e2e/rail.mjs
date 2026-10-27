// web's right rail (the top bar's activity button) at phone width. Ann offers a personal
// service with one activity: it shows under Project Activity & Events with its signups and
// the button's badge; clearing it moves it to Cleared until restored, and opening it goes to
// the activity's sheet. Bob joins Ann's productive project; her edit shows under Votes to
// cast on his rail, Assess opens its vote, and once he votes it moves to History.
//   node tests/e2e/rail.mjs http://localhost:4174 [screens dir]
// It signs up two test accounts, so it may run against the live test network too.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, device, errors, log, rail, railButton, railShows, signUp } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const service = `Bike fixing ${stamp}`;
const activity = `Saturday repairs ${stamp}`;
const coop = `Tool library ${stamp}`;

async function create(page, title, personal) {
  await page.goto(`${BASE}/create/project`);
  if (personal) await page.getByRole('radio', { name: /Personal service/ }).click();
  await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByLabel('Title').fill(title);
  await page.getByLabel('Proposal description').fill('Made by the rail test.');
  for (let i = 0; i < 3; i++) await page.getByRole('button', { name: 'Continue' }).click();
  await page.getByRole('button', { name: 'Publish project' }).click();
  await page.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
  return page.url().split('?')[0];
}

const ann = await device({ viewport: phone });
// Signed out, web's rail shows its four sections collapsed and empty.
await ann.goto(BASE);
await railButton(ann).click();
await rail(ann).getByText('Votes to cast').waitFor({ timeout: 60_000 });
assert.equal(
  (await rail(ann).innerText()).replace(/\s+/g, ' ').trim(),
  'Project Activity & Events Help Requests Requests Votes to cast',
);
await ann.waitForTimeout(1000);
await ann.screenshot({ path: `${out}/phone390-rail-signed-out.png` });
await railButton(ann).click();
await signUp(ann, `ann${stamp}`);
const serviceUrl = await create(ann, service, true);
await ann.getByRole('tab', { name: 'Participation' }).click();
await ann.getByRole('button', { name: 'Add activity' }).click();
const next = () => ann.getByRole('button', { name: 'Next', exact: true }).click();
await ann.getByPlaceholder('Activity title').fill(activity);
await next();
await next();
await ann.getByPlaceholder('Place or address').fill('The shed');
await next();
await ann.getByPlaceholder('For example: Intake desk').fill('Helper');
await next();
await ann.getByPlaceholder('What should happen in this activity?').fill('Fix brakes.');
await next();
await ann.getByRole('button', { name: 'Create activity' }).click();
await ann.locator('.agenda-row', { hasText: activity }).waitFor({ timeout: 60_000 });
log('ann added an activity');

const row = () => rail(ann).locator('.activity-row', { hasText: activity });
await railShows(ann, row, 'the activity');
assert.match(await row().innerText(), /The shed/);
assert.match(await row().innerText(), /0 signed up · 1 needed/);
assert.match(await row().innerText(), /Project activity/i);
await ann.locator('.panel-toggle-right .count-badge').waitFor();
await ann.waitForTimeout(1000); // the rail slides in
await ann.screenshot({ path: `${out}/phone390-rail-activity.png` });
log('the rail lists it with its signups, and the button has a badge');

await row().getByRole('button', { name: 'Dismiss activity card' }).click();
await rail(ann).getByText('No activity yet').waitFor({ state: 'attached' });
await rail(ann).getByText('Cleared').click();
await rail(ann).locator('.cleared-row', { hasText: activity }).waitFor();
await rail(ann).getByRole('button', { name: 'Restore all' }).click();
// As on web, a section emptied by clearing stays collapsed when its cards come back.
await rail(ann).getByText('Project Activity & Events').click();
await row().waitFor();
log('clearing moves it to Cleared; Restore all brings it back');

await row().locator('.activity-open-button').click();
await ann.getByRole('dialog', { name: activity }).waitFor({ timeout: 120_000 });
assert.equal(ann.url().split('?')[0], serviceUrl);
await ann.keyboard.press('Escape');
assert.equal(await ann.locator('.panel-toggle-right .count-badge').count(), 0);
log('opening it shows the activity sheet and clears the badge');

const coopUrl = await create(ann, coop, false);
const bob = await device({ viewport: phone });
await signUp(bob, `bob${stamp}`);
await bob.goto(coopUrl);
await bob.getByRole('heading', { name: coop }).waitFor({ timeout: 120_000 });
await bob.getByRole('button', { name: 'Join project' }).click();
await bob.getByRole('button', { name: '2 members' }).waitFor({ timeout: 60_000 });
log('bob joined the co-op');

await ann.goto(coopUrl);
await ann.getByRole('button', { name: 'Edit', exact: true }).click();
await ann.locator('.edit-sheet').getByLabel('Title').fill(`${coop} co-op`);
await ann.getByRole('button', { name: 'Propose edit' }).click();
await ann.locator('.edit-sheet').waitFor({ state: 'hidden', timeout: 60_000 });
log('ann proposed an edit');

const vote = () => rail(bob).locator('.vote-row', { hasText: coop });
await railShows(bob, vote, 'the edit vote');
assert.match(await vote().innerText(), /Vote · edit/i);
assert.match(await vote().innerText(), /Edit request/);
assert.match(await vote().innerText(), /\d+ yes \/ \d+ no/);
await bob.waitForTimeout(1000);
await bob.screenshot({ path: `${out}/phone390-rail-vote.png` });
await vote().getByRole('button', { name: 'Assess' }).click();
await bob.locator('.choice-bar .choice.yes').click({ timeout: 120_000 });
log('bob opened the vote from his rail and voted yes');

const history = () => rail(bob).locator('.history-row', { hasText: 'You voted Yes' });
for (let i = 0; ; i++) {
  await railButton(bob).click();
  await rail(bob).getByText('No votes waiting').waitFor({ state: 'attached', timeout: 60_000 }).catch(() => {});
  if (await rail(bob).getByText('History', { exact: true }).count()) {
    await rail(bob).getByText('History', { exact: true }).click();
    if (await history().waitFor({ timeout: 5_000 }).then(() => true, () => false)) break;
  }
  if (i > 10) throw new Error('the cast vote never reached History');
  await railButton(bob).click();
}
await bob.screenshot({ path: `${out}/phone390-rail-history.png` });
log('the vote left Votes to cast and is in History');

assert.deepEqual(errors, []);
await browser.close();
log('rail ok');
