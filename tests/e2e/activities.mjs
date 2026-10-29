// Activities and roles (ADR 0056) end to end at phone width. A personal service's Activity
// phase is its first, so Ann offers one: she adds an activity with one Helper place on web's
// calendar; Bob joins, takes the role (the row then reads Full), leaves it again; Ann's
// suggestion of Bob for the role shows on his sheet and he declines it (Ann is told).
// Project chats take photos since the same change: Bob sends one and Ann sees it load. Last,
// a two-minute activity Bob takes part in moves to Past activity, where he checks in and
// rates it.
//   node tests/e2e/activities.mjs http://localhost:4174 [screens dir]
// It signs up two test accounts, so it may run against the live test network too.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, device, errors, log, signUp, continueSteps } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const title = `Bike fixing ${stamp}`;
const activity = `Saturday repairs ${stamp}`;

const ann = await device({ viewport: phone });
await signUp(ann, `ann${stamp}`);
await ann.goto(`${BASE}/create/project`);
await ann.getByRole('radio', { name: /Personal service/ }).click();
await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('Proposal description').fill('I fix bikes on weekends.');
await continueSteps(ann, 3);
await ann.getByRole('button', { name: 'Publish project' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
const projectUrl = ann.url().split('?')[0];
log('ann offers a personal service');

/** Reload Notifications until a card with `text` shows (derived from the network). */
async function notified(page, text) {
  const card = page.locator('.notification-card', { hasText: text }).first();
  for (let i = 0; i < 20; i++) {
    await page.goto(`${BASE}/notifications`);
    await page.getByRole('heading', { name: 'Notifications' }).waitFor({ timeout: 60_000 });
    if (await card.waitFor({ timeout: 10_000 }).then(() => true, () => false)) return card;
  }
  throw new Error(`timed out waiting for a "${text}" notification`);
}

const takePart = async (page) => {
  await page.goto(projectUrl);
  await page.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
  await page.getByRole('tab', { name: 'Participation' }).click();
};

await takePart(ann);
await ann.getByRole('button', { name: 'Add activity' }).click();
const next = () => ann.getByRole('button', { name: 'Next', exact: true }).click();
await ann.getByPlaceholder('Activity title').fill(activity);
await next();
await next(); // the next hour, web's default
await ann.getByPlaceholder('Place or address').fill('The shed');
await next();
await ann.getByPlaceholder('For example: Intake desk').fill('Helper');
await ann.getByPlaceholder('No cap').fill('1');
await next();
await ann.getByPlaceholder('What should happen in this activity?').fill('Fix brakes and chains.');
await next();
await ann.screenshot({ path: `${out}/phone390-activity-review.png` });
await ann.getByRole('button', { name: 'Create activity' }).click();
const row = (page) => page.locator('.agenda-row', { hasText: activity });
await row(ann).waitFor({ timeout: 60_000 });
assert.equal(await row(ann).locator('.agenda-chip').innerText(), '1 more needed');
await ann.screenshot({ path: `${out}/phone390-activity-agenda.png` });
log('ann added an activity; its row needs one more');

const bob = await device({ viewport: phone });
await signUp(bob, `bob${stamp}`);
await bob.goto(projectUrl);
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await bob.getByRole('button', { name: 'Join project' }).click();
await bob.getByRole('button', { name: '2 members' }).waitFor({ timeout: 60_000 });
for (let i = 0; ; i++) {
  await takePart(bob);
  if (await row(bob).waitFor({ timeout: 20_000 }).then(() => true, () => false)) break;
  if (i > 8) throw new Error('the activity never reached bob');
}
await row(bob).click();
const sheet = bob.getByRole('dialog', { name: activity });
await sheet.getByRole('button', { name: 'Take role' }).click();
await sheet.getByRole('button', { name: 'Leave role' }).waitFor({ timeout: 60_000 });
await sheet.getByText('1 of 1 filled').waitFor();
await bob.screenshot({ path: `${out}/phone390-activity-sheet.png` });
await bob.keyboard.press('Escape');
assert.equal(await row(bob).locator('.agenda-chip').innerText(), 'Full');
log('bob took the helper role; the row reads Full');
await notified(ann, `took the "Helper" role in ${activity}`);
log('ann is told bob took the role');

await row(bob).click();
await sheet.getByRole('button', { name: 'Leave role' }).click();
await sheet.getByRole('button', { name: 'Take role' }).waitFor({ timeout: 60_000 });
await bob.keyboard.press('Escape');
log('bob left the role');

// Ann suggests Bob for the role; Bob sees it on his sheet and declines.
await takePart(ann);
await row(ann).click();
const annSheet = ann.getByRole('dialog', { name: activity });
await annSheet.getByRole('button', { name: 'Suggest someone' }).click();
await annSheet.getByPlaceholder('Search username').fill(`bob${stamp}`);
await annSheet.getByRole('button', { name: `bob${stamp}` }).click();
await annSheet.getByText(`Suggested · bob${stamp}`).waitFor({ timeout: 60_000 });
log('ann suggested bob');
for (let i = 0; ; i++) {
  await takePart(bob);
  await row(bob).click();
  if (await sheet.getByText(`Suggested · bob${stamp}`).waitFor({ timeout: 15_000 }).then(() => true, () => false)) break;
  if (i > 8) throw new Error('the suggestion never reached bob');
}
await notified(bob, `You've been suggested for Helper on ${activity}.`);
log('bob is told about the suggestion');
await takePart(bob);
await row(bob).click();
await sheet.getByRole('button', { name: 'Decline' }).click();
await sheet.getByText(`Suggested · bob${stamp}`).waitFor({ state: 'detached', timeout: 60_000 });
log('bob declined the suggestion');
await notified(ann, 'The person you suggested declined this role.');
log('ann is told bob declined');

// Photos in the project chat (ADR 0056 ride-along).
const png = Buffer.from(
  await bob.evaluate(() => {
    const canvas = Object.assign(document.createElement('canvas'), { width: 160, height: 120 });
    const g = canvas.getContext('2d');
    g.fillStyle = '#4a7c3f';
    g.fillRect(0, 0, 160, 120);
    return canvas.toDataURL('image/png').split(',')[1];
  }),
  'base64',
);
const caption = `Chain after ${stamp}`;
await bob.goto(`${projectUrl}?tab=chat`);
await bob.locator('input.attach-input[accept]').setInputFiles({ name: 'chain.png', mimeType: 'image/png', buffer: png });
await bob.locator('.attach-preview img').waitFor();
await bob.getByRole('textbox', { name: 'Message the project...' }).fill(caption);
await bob.getByRole('button', { name: 'Send', exact: true }).click();
await bob.locator('.chat-log').getByText('Sending…').waitFor({ state: 'detached', timeout: 120_000 });
const photoIn = (page) => page.locator('.chat-message', { hasText: caption }).locator('.message-photo');
for (let i = 0; ; i++) {
  await ann.goto(`${projectUrl}?tab=chat`);
  if (await photoIn(ann).waitFor({ timeout: 20_000 }).then(() => true, () => false)) break;
  if (i > 8) throw new Error('the photo never reached ann');
}
await ann.waitForFunction((el) => el.complete && el.naturalWidth > 0, await photoIn(ann).elementHandle(), {
  timeout: 120_000,
});
await ann.screenshot({ path: `${out}/phone390-project-chat-photo.png` });
log('bob sent a photo in the project chat; ann sees it');

// Past activity: a short activity ends; Bob checks in Completed and rates it 4 stars.
const local = (d) => {
  const pad = (n) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}`;
};
const quick = `Quick tune ${stamp}`;
const start = new Date(Date.now() + 2 * 60_000);
const end = new Date(start.getTime() + 60_000);
await takePart(ann);
await ann.getByRole('button', { name: 'Add activity' }).click();
await ann.getByPlaceholder('Activity title').fill(quick);
await next();
await ann.getByLabel('Start time').fill(local(start));
await ann.getByLabel('Finish time').fill(local(end));
await next();
await ann.getByPlaceholder('Place or address').fill('The shed');
await next();
await ann.getByPlaceholder('For example: Intake desk').fill('Helper');
await next();
await ann.getByPlaceholder('What should happen in this activity?').fill('A quick tune-up.');
await next();
await ann.getByRole('button', { name: 'Create activity' }).click();
const quickRow = (page) => page.locator('.agenda-row', { hasText: quick });
await quickRow(ann).waitFor({ timeout: 60_000 });
for (let i = 0; ; i++) {
  await takePart(bob);
  if (await quickRow(bob).waitFor({ timeout: 20_000 }).then(() => true, () => false)) break;
  if (i > 8) throw new Error('the short activity never reached bob');
}
await quickRow(bob).click();
const quickSheet = bob.getByRole('dialog', { name: quick });
await quickSheet.getByRole('button', { name: 'Take role' }).click();
await quickSheet.getByRole('button', { name: 'Leave role' }).waitFor({ timeout: 60_000 });
await bob.keyboard.press('Escape');
log('bob took a role in a two-minute activity');
await bob.waitForTimeout(Math.max(0, end.getTime() - Date.now()) + 5_000);
await takePart(bob);
await bob.getByText('Past activity').click();
await quickRow(bob).click();
await quickSheet.getByRole('button', { name: 'Completed', exact: true }).click();
await quickSheet.getByText('1/1 completed · 0 uncompleted').waitFor({ timeout: 60_000 });
await quickSheet.getByRole('button', { name: '4★' }).click();
await quickSheet.getByPlaceholder('Optional comment').fill('Smooth gears.');
await quickSheet.getByRole('button', { name: 'Save rating' }).click();
await quickSheet.getByText('4★ · 1 ratings').waitFor({ timeout: 60_000 });
await bob.screenshot({ path: `${out}/phone390-activity-history.png` });
log('bob checked in and rated the past activity');

assert.deepEqual(errors, [], 'no page errors');
log('all activity checks passed');
await browser.close();
