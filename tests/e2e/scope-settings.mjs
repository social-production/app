// web's scope Settings and Vote sheets (ADR 0068 part e). Founder one makes a channel and
// founder two joins it; at 390px two proposes a new description from Settings, one finds the
// Vote button, votes Yes, and the description changes; Decisions lists it as passed.
//   node tests/e2e/scope-settings.mjs http://localhost:4174 [screens dir]
// It uses the dev founders (founders on the live test network too) and makes one channel.
import assert from 'node:assert/strict';
import { mkdirSync, readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const handle = `settings${stamp}`;
const before = 'Made by the settings test.';
const after = `A better description ${stamp}.`;
const [one, two] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));

const a = await device({ viewport: phone });
await phraseSignIn(a, one.phrase.split(' '));
await a.goto(`${BASE}/create/channel`);
await a.getByLabel(/Channel handle/).fill(handle);
await a.getByLabel(/Description/).fill(before);
await a.getByRole('button', { name: 'Create channel' }).click();
await a.getByRole('heading', { name: handle }).waitFor({ timeout: 120_000 });
const url = a.url();
log('one made the channel');

// Two joins (the index lists a new channel within a few minutes).
const b = await device({ viewport: phone });
await phraseSignIn(b, two.phrase.split(' '));
const heading = b.getByRole('heading', { name: handle });
for (let i = 0; ; i++) {
  await b.goto(url);
  if (await heading.waitFor({ timeout: 15_000 }).then(() => true, () => false)) break;
  if (i >= 20) throw new Error('the channel never reached the index');
}
await b.getByRole('button', { name: `Join ${handle}` }).click();
await b.getByRole('button', { name: `Leave ${handle}` }).waitFor({ timeout: 120_000 });
log('two joined');

// Two proposes a new description; the name is the handle and can't change.
await b.getByRole('button', { name: 'Settings', exact: true }).click();
const settings = b.locator('.scope-settings');
assert.equal(await settings.getByLabel('Name').isDisabled(), true, 'the handle stays');
await settings.getByLabel('Description').fill(after);
await b.screenshot({ path: `${out}/phone390-scope-settings.png` });
await settings.getByRole('button', { name: 'Propose change' }).click();
await settings.getByText('Vote started').waitFor({ timeout: 60_000 });
const open = settings.locator('.decision-card.open', { hasText: 'Change: new description' });
await open.waitFor({ timeout: 60_000 });
await b.screenshot({ path: `${out}/phone390-scope-decisions.png` });
await settings.locator('.sheet-close').click();
log('two proposed the change');

// One sees the Vote button and votes it through.
const fab = a.getByRole('button', { name: /^Vote: \d+ waiting/ });
for (let i = 0; ; i++) {
  await a.goto(url);
  await a.getByRole('heading', { name: handle }).waitFor({ timeout: 60_000 });
  if (await fab.waitFor({ timeout: 15_000 }).then(() => true, () => false)) break;
  if (i >= 20) throw new Error('the Vote button never showed');
}
await fab.click();
const vote = a.locator('.scope-vote');
await vote.getByText('Change: new description').waitFor();
await vote.getByText(after).waitFor();
await a.screenshot({ path: `${out}/phone390-scope-vote.png` });
await vote.getByRole('button', { name: 'Yes', exact: true }).click();
await vote.getByText('You voted Yes.').waitFor({ timeout: 60_000 });
await vote.waitFor({ state: 'detached', timeout: 30_000 });
log('one voted');

// The change applies once the vote passes; Decisions lists it as passed.
for (let i = 0; ; i++) {
  await a.goto(url);
  if (await a.getByText(after).waitFor({ timeout: 15_000 }).then(() => true, () => false)) break;
  if (i >= 20) throw new Error('the new description never showed');
}
await a.getByRole('button', { name: 'Settings', exact: true }).click();
await a.locator('.scope-settings').getByRole('tab', { name: /Decisions/ }).click();
await a
  .locator('.scope-settings .decision-card.approved', { hasText: 'Change: new description' })
  .waitFor({ timeout: 60_000 });
log('the description changed and the decision passed');

assert.deepEqual(errors, []);
await browser.close();
log('scope-settings: ok');
