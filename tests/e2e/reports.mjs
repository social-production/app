// Reports end to end (ADR 0071), at 390px. A new person publishes a post. Founder one opens it,
// finds web's ⋮ menu (Copy link, Report post), reports it as spam and sees "Under review".
// Founder two opens the same ⋮, sees the report with its thresholds and votes Yes (two of the
// three votes a small audience needs). The author's own ⋮ says they can't report themselves,
// and their profile's post card shows "Under review" once the index has re-read it.
//   node tests/e2e/reports.mjs http://localhost:4174 [screenshot dir]
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn, signUp } from './lib.mjs';

const [one, two] = JSON.parse(
  readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)),
);
const stamp = Date.now().toString(36).slice(-4);
const body = `Cheap pills here ${stamp}`;
const shots = process.argv[3];
const phone = { viewport: { width: 390, height: 844 } };

const bob = await device(phone);
await signUp(bob, `Poster${stamp}`);
await bob.goto(BASE);
await bob.getByRole('button', { name: 'Create', exact: true }).click();
await bob.getByRole('menuitem', { name: /Post/ }).click();
await bob.getByLabel(/What's on your mind/).fill(body);
await bob.getByRole('button', { name: 'Publish post' }).click();
await bob.locator('.linked-body.detail', { hasText: body }).waitFor({ timeout: 120_000 });
const url = bob.url();
assert.match(url, /\/posts\//);
const profile = await bob.locator('.discussion-page .header-name').getAttribute('href');
log('a new person published a post');

// The author's own menu: Copy link, and Report says they can't.
await bob.getByRole('button', { name: 'Report post' }).click();
const own = bob.getByRole('dialog', { name: 'Options' });
await own.getByRole('menuitem', { name: 'Copy link' }).waitFor();
await own.getByRole('menuitem', { name: 'Report post' }).click();
await own.getByText("You can't report yourself").waitFor();
await own.getByRole('button', { name: 'Close' }).click();
log("the author's ⋮ offers Copy link and says they can't report themselves");

const ann = await device(phone);
await phraseSignIn(ann, one.phrase.split(' '));
await ann.goto(url);
await ann.locator('.linked-body.detail', { hasText: body }).waitFor({ timeout: 120_000 });
await ann.getByRole('button', { name: 'Report post' }).click();
await ann
  .getByRole('dialog', { name: 'Options' })
  .getByRole('menuitem', { name: 'Report post' })
  .click();
const composer = ann.locator('.report-composer');
await composer.getByLabel('Reason').selectOption('spam');
await composer.getByLabel("What's wrong?").fill('Adverts for pills.');
if (shots) await ann.screenshot({ path: `${shots}/report-composer.png` });
await composer.getByRole('button', { name: 'Submit report' }).click();
await ann
  .locator('.discussion-page .report-trigger .status-token', {
    hasText: 'Under review',
  })
  .waitFor({ timeout: 60_000 });
log('founder one reported it: Under review');

const cat = await device(phone);
await phraseSignIn(cat, two.phrase.split(' '));
for (let i = 0; ; i++) {
  await cat.goto(url);
  await cat.locator('.linked-body.detail', { hasText: body }).waitFor({ timeout: 120_000 });
  const trigger = cat.locator('.discussion-page .report-trigger');
  if ((await trigger.innerText()).includes('Under review')) break;
  if (i >= 20) throw new Error('founder two never saw the report');
  await cat.waitForTimeout(3000);
}
await cat.getByRole('button', { name: /View post report/ }).click();
const review = cat.getByRole('dialog', { name: 'Under review' });
await review.getByText('Adverts for pills.').waitFor();
await review.getByText(/^Delete at 3 votes cast · 66% yes/).waitFor();
await review.getByText('Current: 1 yes · 0 no · 1 total').waitFor();
if (shots) await cat.screenshot({ path: `${shots}/report-review.png` });
await review.getByRole('button', { name: 'Yes', exact: true }).click();
await cat.waitForTimeout(500);
for (let i = 0; ; i++) {
  const trigger = cat.locator('.discussion-page .report-trigger');
  if (!(await cat.locator('.report-menu').count())) {
    await trigger.click();
  }
  const text = await cat.locator('.report-menu').innerText();
  if (text.includes('Current: 2 yes · 0 no · 2 total')) break;
  // A refused vote reopens the menu with the reason.
  if (i >= 30 || (await cat.locator('.report-menu [role="alert"]').count())) {
    throw new Error(`founder two's vote never counted; the menu says: ${text}`);
  }
  await cat.locator('.report-menu .menu-dismiss').click();
  await cat.waitForTimeout(2000);
}
log('founder two saw the thresholds and voted yes: 2 of 3');

for (let i = 0; ; i++) {
  await bob.goto(`${BASE}${profile}`);
  const card = bob.locator('.feed-stack .personal-post-card', {
    hasText: body,
  });
  const flagged = card.locator('.status-token', { hasText: 'Under review' });
  if (
    await flagged.waitFor({ timeout: 10_000 }).then(
      () => true,
      () => false,
    )
  )
    break;
  if (i >= 30) throw new Error('the profile card never showed Under review');
  await bob.waitForTimeout(5000);
}
if (shots) await bob.screenshot({ path: `${shots}/report-card.png` });
log('the profile card shows Under review');

assert.deepEqual(errors, []);
await browser.close();
log('reports: ok');
