// Bot marks end to end (ADR 0074), at 390px. A new person signs up. Founder one opens their
// profile, taps web's Bot action, confirms "Mark … as a possible bot?" and sees "Marked bot";
// the standing chip opens the Trust sheet, whose Bot marks tab lists the founder. One mark
// opens no review (that takes three full members, more than dev-founders has), so no review
// card shows. Clearing the mark takes the founder off the list again.
//   node tests/e2e/bot-reviews.mjs http://localhost:4174 [screenshot dir]
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn, signUp } from './lib.mjs';

const [one] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const stamp = Date.now().toString(36).slice(-4);
const shots = process.argv[3];
const phone = { viewport: { width: 390, height: 844 } };
const name = `Maybe${stamp}`;

/** The agent key from Settings (on a phone the Settings link sits in the More sheet). */
async function keyOf(page) {
  await page.goto(`${BASE}/settings`);
  const key = page.locator('#settings-account .key');
  await key.waitFor({ timeout: 60_000 });
  return (await key.innerText()).trim();
}

const bob = await device(phone);
await signUp(bob, name);
const profile = `${BASE}/profile/${await keyOf(bob)}`;
log('a new person signed up');

const ann = await device(phone);
await phraseSignIn(ann, one.phrase.split(' '));
for (let i = 0; ; i++) {
  await ann.goto(profile);
  const seen = await ann
    .getByRole('heading', { name })
    .waitFor({ timeout: 15_000 })
    .then(
      () => true,
      () => false,
    );
  if (seen) break;
  if (i >= 12) throw new Error('the new profile never loaded');
}

const bot = ann.getByRole('button', { name: `Mark ${name} as a bot` });
await bot.waitFor({ timeout: 60_000 });
await bot.click();
const confirm = ann.getByRole('alertdialog');
await confirm.getByText(`Mark ${name} as a possible bot?`).waitFor();
if (shots) await ann.screenshot({ path: `${shots}/bot-confirm.png` });
await confirm.getByRole('button', { name: 'Mark as bot' }).click();
const marked = ann.getByRole('button', { name: `Clear bot mark for ${name}` });
await marked.waitFor({ timeout: 60_000 });
assert.equal(await marked.locator('.action-text').innerText(), 'Marked bot');
assert.equal(await ann.locator('.bot-review-card').count(), 0, 'one mark opens no review');
log('founder one marked them: "Marked bot", no review yet');

await ann.getByRole('button', { name: /^Standing: / }).click();
const trust = ann.getByRole('dialog', { name: 'Trust' });
await trust.getByRole('tab', { name: /Bot marks/ }).click();
await trust.locator('.person-row').first().waitFor({ timeout: 30_000 });
assert.equal(await trust.locator('.person-row').count(), 1);
if (shots) await ann.screenshot({ path: `${shots}/bot-trust-sheet.png` });
await ann.keyboard.press('Escape');
log('the Trust sheet lists the mark');

await marked.click();
await ann
  .getByRole('button', { name: `Mark ${name} as a bot` })
  .waitFor({ timeout: 60_000 });
log('clearing the mark turns it back into Bot');

assert.deepEqual(errors, []);
await browser.close();
log('bot-reviews: ok');
