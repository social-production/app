// Your activity (the top bar's activity button, Screen Review §1) at phone width. A new member
// sees its three tabs and Coming up's empty state. The dev founders move Ann's event to its
// Activity phase and she adds an activity with one Helper place: it shows under Coming up with
// its signups and the button's badge; clearing it moves it to Cleared until restored, and
// opening it goes to the activity's sheet. Bob joins Ann's project; her edit shows under Votes
// on his rail, Assess opens its vote, and once he votes it moves to History.
//   node tests/e2e/rail.mjs http://localhost:4174 [screens dir]
// It needs the dev founders, so it runs only against a local seed node.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, device, errors, log, rail, railButton, railShows, railTab, railTabs, signUp, continueSteps } from './lib.mjs';
import { advanceTo, approvePlan, createEvent, daysFromNow, filePlan, signInFounders, until } from './event-flow.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const title = `Bike fixing ${stamp}`;
const activity = `Saturday repairs ${stamp}`;
const coop = `Tool library ${stamp}`;

async function create(page, name) {
  await page.goto(`${BASE}/create/project`);
  await page.getByLabel('Title').fill(name);
  await page.getByLabel('What do you propose?').fill('Made by the rail test.');
  await continueSteps(page, 3);
  await page.getByRole('button', { name: 'Publish proposal' }).click();
  await page.getByRole('heading', { name }).waitFor({ timeout: 120_000 });
  return page.url().split('?')[0];
}

// A new member's Your activity shows its three tabs and Coming up's empty state.
const bob = await device({ viewport: phone });
await bob.goto(BASE);
await signUp(bob, `bob${stamp}`);
await railButton(bob).click();
await rail(bob).getByText('No activity yet').waitFor({ timeout: 60_000 });
assert.deepEqual(
  (await railTabs(bob).allInnerTexts()).map((t) => t.trim().replace(/\s*\d+$/, '')),
  ['Coming up', 'Votes', 'Requests'],
);
await bob.waitForTimeout(1000);
await bob.screenshot({ path: `${out}/phone390-rail-empty.png` });
await railButton(bob).click();
log('a new member sees the three tabs and the empty state');

const { ann, bob: voter } = await signInFounders(phone);
const eventUrl = await createEvent(ann, voter, title);
await advanceTo(ann, voter, 'Plan');
const planTitle = `Repair fortnight ${stamp}`;
await filePlan(ann, { title: planTitle, range: [daysFromNow(0), daysFromNow(14)], start: '00:00', finish: '23:59', place: 'The shed' });
await approvePlan(ann, voter, planTitle);
await advanceTo(ann, voter, 'Activity');
await voter.context().close();
await ann.goto(eventUrl);
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await ann.getByRole('tab', { name: 'Participation' }).click();
await ann.getByRole('button', { name: 'Add activity' }).click();
const next = () => ann.getByRole('button', { name: 'Next', exact: true }).click();
await ann.getByPlaceholder('Activity title').fill(activity);
await next();
await next();
await ann.getByPlaceholder('Search a place').fill('The shed');
await next();
await ann.getByPlaceholder('For example: Intake desk').fill('Helper');
await ann.getByPlaceholder('No cap').fill('1');
await next();
await ann.getByPlaceholder('What should happen in this activity?').fill('Fix brakes and chains.');
await next();
await ann.getByRole('button', { name: 'Create activity' }).click();
await ann.locator('.agenda-row', { hasText: activity }).waitFor({ timeout: 60_000 });
log('ann added an activity');

// The rail lists the four soonest activities, and the founders keep earlier runs' ones, so
// Bob (new, nothing else coming up) joins the event and watches his rail.
await bob.goto(eventUrl);
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await bob.getByRole('button', { name: 'Join event' }).click();
await bob.getByRole('button', { name: '3 members' }).waitFor({ timeout: 60_000 });
log('bob joined the event');

const row = () => rail(bob).locator('.activity-row', { hasText: activity });
await railShows(bob, row, 'the activity');
assert.match(await row().innerText(), /The shed/);
assert.match(await row().innerText(), /0 signed up · 1 needed/);
await bob.locator('.panel-toggle-right .count-badge').waitFor();
await bob.waitForTimeout(1000); // the rail slides in
await bob.screenshot({ path: `${out}/phone390-rail-activity.png` });
log('the rail lists it with its signups, and the button has a badge');

await row().getByRole('button', { name: 'Dismiss activity card' }).click();
await rail(bob).getByText('No activity yet').waitFor({ state: 'attached' });
await rail(bob).getByText('Cleared').click();
await rail(bob).locator('.cleared-row', { hasText: activity }).waitFor();
await rail(bob).getByRole('button', { name: 'Restore all' }).click();
await railTab(bob, 'Coming up');
await row().waitFor();
log('clearing moves it to Cleared; Restore all brings it back');

await row().locator('.activity-open-button').click();
await bob.getByRole('dialog', { name: activity }).waitFor({ timeout: 120_000 });
assert.equal(bob.url().split('?')[0].split('/').pop(), eventUrl.split('/').pop());
await bob.keyboard.press('Escape');
assert.equal(await bob.locator('.panel-toggle-right .count-badge').count(), 0);
log('opening it shows the activity sheet and clears the badge');

const coopUrl = await create(ann, coop);
await bob.goto(coopUrl);
await bob.getByRole('heading', { name: coop }).waitFor({ timeout: 120_000 });
await bob.getByRole('button', { name: 'Join project' }).click();
await bob.getByRole('button', { name: '2 members' }).waitFor({ timeout: 60_000 });
log('bob joined the co-op');

await ann.goto(coopUrl);
await until(ann, () => ann.getByRole('button', { name: '2 members' }), 'bob joining');
await ann.getByRole('button', { name: 'More actions' }).click();
await ann.getByRole('menuitem', { name: 'Edit' }).click();
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
