// Reports on comments and chat messages end to end (ADR 0071 part b), at 390px.
// A new person publishes a post and comments spam on it. Founder one reports the comment from
// its ⋮ (web's "Report comment") and sees "Under review"; founder two finds the report and
// votes Yes (two of three). Then founder one creates a project, the new person posts in its
// chat, and founder one reports that message from the bubble menu as serious harm: the bubble
// hides behind web's "Hidden for serious harm report — reveal to read", and founder two gets
// "Vote to remove" / "Vote to keep" in the bubble menu.
//   node tests/e2e/message-reports.mjs http://localhost:4174 [screenshot dir]
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { BASE, browser, continueSteps, device, errors, log, phraseSignIn, signUp } from './lib.mjs';

const [one, two] = JSON.parse(
  readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)),
);
const stamp = Date.now().toString(36).slice(-4);
const spam = `Cheap pills in the comments ${stamp}`;
const chatSpam = `Pills for sale in chat ${stamp}`;
const shots = process.argv[3];
const phone = { viewport: { width: 390, height: 844 } };
const HIDDEN = 'Hidden for serious harm report — reveal to read';

const bob = await device(phone);
await signUp(bob, `Spammer${stamp}`);
await bob.goto(BASE);
await bob.getByRole('button', { name: 'Create', exact: true }).click();
await bob.getByRole('menuitem', { name: /Post/ }).click();
await bob.getByLabel(/What's on your mind/).fill(`A quiet post ${stamp}`);
await bob.getByRole('button', { name: 'Publish post' }).click();
await bob.locator('.linked-body.detail').waitFor({ timeout: 120_000 });
const postUrl = bob.url();
assert.match(postUrl, /\/posts\//);
const box = bob.getByLabel('Write a comment...');
await box.fill(spam);
await box.press('Enter');
await bob.locator('.comment-card', { hasText: spam }).waitFor({ timeout: 60_000 });
log('a new person commented spam on their post');

/** Open `url` until the comment shows (a new post can take a while to reach a node). */
async function openComment(page, url) {
  for (let i = 0; ; i++) {
    await page.goto(url);
    const card = page.locator('.comment-card', { hasText: spam });
    if (await card.waitFor({ timeout: 20_000 }).then(() => true, () => false)) return card;
    if (i >= 10) throw new Error('the comment never showed');
  }
}

const ann = await device(phone);
await phraseSignIn(ann, one.phrase.split(' '));
const annComment = await openComment(ann, postUrl);
await annComment.getByRole('button', { name: 'Report comment' }).click();
await ann
  .getByRole('dialog', { name: 'Options' })
  .getByRole('menuitem', { name: 'Report comment' })
  .click();
const composer = ann.locator('.report-composer');
await composer.getByLabel("What's wrong?").fill('Adverts for pills.');
await composer.getByRole('button', { name: 'Submit report' }).click();
await annComment
  .locator('.report-trigger .status-token', { hasText: 'Under review' })
  .waitFor({ timeout: 60_000 });
if (shots) await ann.screenshot({ path: `${shots}/comment-report.png` });
log('founder one reported the comment: Under review');

const cat = await device(phone);
await phraseSignIn(cat, two.phrase.split(' '));
for (let i = 0; ; i++) {
  const card = await openComment(cat, postUrl);
  const trigger = card.locator('.report-trigger').first();
  if ((await trigger.innerText()).includes('Under review')) {
    await trigger.click();
    break;
  }
  if (i >= 20) throw new Error('founder two never saw the comment report');
  await cat.waitForTimeout(3000);
}
const review = cat.getByRole('dialog', { name: 'Under review' });
await review.getByText('Adverts for pills.').waitFor();
await review.getByText('Current: 1 yes · 0 no · 1 total').waitFor();
await review.getByRole('button', { name: 'Yes', exact: true }).click();
await cat
  .locator('.comment-card', { hasText: spam })
  .locator('.report-trigger')
  .first()
  .click();
await cat.getByText('Current: 2 yes · 0 no · 2 total').waitFor({ timeout: 30_000 });
await cat.locator('.report-menu .menu-dismiss').click();
log('founder two voted yes on the comment: 2 of 3');

// A project chat: the bubble menu's Report, as on web's ProjectChatTab.
const title = `Pill free garden ${stamp}`;
await ann.goto(`${BASE}/create/project`);
await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('Proposal description').fill('No adverts here.');
await continueSteps(ann, 3);
await ann.getByRole('button', { name: 'Publish project' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
const projectUrl = ann.url();

/** Open the project's Chat tab until `text` shows in its log. */
async function openChat(page, text) {
  for (let i = 0; i < 20; i++) {
    await page.goto(projectUrl);
    await page.locator('.project-detail h1').waitFor({ timeout: 60_000 });
    await page.getByRole('tab', { name: 'Chat' }).click();
    const found = page.locator('.chat-log .chat-message', { hasText: text }).first();
    if (await found.waitFor({ timeout: 12_000 }).then(() => true, () => false)) return found;
  }
  throw new Error(`the chat never showed ${text}`);
}

await bob.goto(projectUrl);
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
await bob.getByRole('tab', { name: 'Chat' }).click();
await bob.getByRole('textbox', { name: 'Message the project...' }).fill(chatSpam);
await bob.getByRole('button', { name: 'Send', exact: true }).click();
await bob.locator('.chat-log').getByText(chatSpam).first().waitFor({ timeout: 30_000 });
log('the new person posted in the project chat');

const annBubble = await openChat(ann, chatSpam);
await annBubble.locator('.message-time').first().click();
await ann.getByRole('menuitem', { name: 'Report', exact: true }).click();
const chatComposer = ann.locator('.report-composer');
await chatComposer.getByLabel('Reason').selectOption('serious-harm');
await chatComposer.getByLabel("What's wrong?").fill('Selling drugs.');
await chatComposer.getByRole('button', { name: 'Submit report' }).click();
const hidden = ann.locator('.chat-log .hidden-toggle', { hasText: HIDDEN });
await hidden.waitFor({ timeout: 60_000 });
assert.equal(await ann.locator('.chat-log').getByText(chatSpam).count(), 0, 'the text is hidden');
await hidden.click();
await ann.locator('.chat-log').getByText(chatSpam).first().waitFor();
if (shots) await ann.screenshot({ path: `${shots}/chat-report.png` });
log('founder one reported the message: hidden, and revealed on request');

for (let i = 0; ; i++) {
  await cat.goto(projectUrl);
  await cat.locator('.project-detail h1').waitFor({ timeout: 60_000 });
  await cat.getByRole('tab', { name: 'Chat' }).click();
  const toggle = cat.locator('.chat-log .hidden-toggle', { hasText: HIDDEN });
  if (await toggle.waitFor({ timeout: 15_000 }).then(() => true, () => false)) break;
  if (i >= 20) throw new Error('founder two never saw the hidden message');
}
await cat
  .locator('.chat-log .chat-message', { has: cat.locator('.hidden-toggle') })
  .locator('.message-time')
  .first()
  .click();
await cat.getByRole('menuitem', { name: 'Vote to keep' }).waitFor();
await cat.getByRole('menuitem', { name: 'Vote to remove' }).click();
await cat.waitForTimeout(3000);
assert.equal(await cat.locator('.chat-panel [role="alert"]').count(), 0, 'the vote was taken');
log('founder two voted to remove from the bubble menu');

assert.deepEqual(errors, []);
await browser.close();
log('message reports: ok');
