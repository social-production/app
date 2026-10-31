// Threads and posts end to end (ADR 0070). Founder one starts a thread tagged with the test
// channel and comments on it. A new person finds web's ThreadCard in the feed, votes the thread
// up, replies to the comment (nested under it) and votes the comment up. They then publish a
// post, which shows on their profile on web's PersonalPostCard. Founder one sees the reply and
// the comment's vote.
//   node tests/e2e/threads.mjs http://localhost:4174 [screenshot dir]
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { BASE, browser, device, errors, log, phraseSignIn, signUp, tagTestChannel } from './lib.mjs';

const [one] = JSON.parse(readFileSync(new URL('./fixtures/dev-founders.json', import.meta.url)));
const stamp = Date.now().toString(36).slice(-4);
const title = `Which seeds for spring ${stamp}`;
const shots = process.argv[3];

const ann = await device({ viewport: { width: 390, height: 844 } });
await phraseSignIn(ann, one.phrase.split(' '));
await ann.goto(BASE);
await ann.getByRole('button', { name: 'Create', exact: true }).click();
await ann.getByRole('menuitem', { name: /Thread/ }).click();
await ann.getByLabel(/^Thread title/).fill(title);
await ann.getByLabel(/^Opening post/).fill('Share what grew well last year.');
await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByLabel('Primary tag type').waitFor();
await tagTestChannel(ann);
await ann.getByRole('button', { name: 'Publish thread' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
assert.match(ann.url(), /\/threads\//);
const url = ann.url();
const composer = ann.getByLabel('Write a comment...');
await composer.fill('Tomatoes did best for me.');
await composer.press('Enter');
const annComment = ann.locator('.comment-card', { hasText: 'Tomatoes did best' });
await annComment.waitFor({ timeout: 60_000 });
await ann.getByText('1 comment').waitFor();
if (shots) await ann.screenshot({ path: `${shots}/thread.png` });
log('founder one started a thread and commented');

const bob = await device({ viewport: { width: 390, height: 844 } });
await signUp(bob, `Reader${stamp}`);
for (let i = 0; ; i++) {
  await bob.goto(BASE);
  await bob.getByRole('button', { name: 'Sort public feed by' }).click();
  await bob.getByRole('menuitemradio', { name: 'Most recent' }).click();
  await bob.locator('.surface').first().waitFor({ timeout: 60_000 }).catch(() => {});
  const card = bob.locator('.thread-card', { hasText: title });
  if (await card.count()) {
    await card.getByText('Share what grew well').waitFor();
    break;
  }
  if (i >= 40) throw new Error('the thread never reached the feed');
  await bob.waitForTimeout(3000);
}
log('the feed shows its ThreadCard');

await bob.goto(url);
await bob.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
const overview = bob.locator('.overview-shell');
await overview.getByRole('button', { name: 'Vote up' }).click();
await overview.locator('.vote-count', { hasText: '1' }).waitFor();
const comment = bob.locator('.comment-card', { hasText: 'Tomatoes did best' });
await comment.waitFor({ timeout: 60_000 });
await comment.getByRole('button', { name: 'Vote up' }).first().click();
await comment.locator('.vote-count').first().filter({ hasText: '1' }).waitFor();
await comment.getByRole('button', { name: 'Reply' }).first().click();
const reply = bob.getByLabel('Write a reply...');
await reply.fill('Which kind of tomato?');
await reply.press('Enter');
await comment.locator('.reply-stack .comment-card', { hasText: 'Which kind' }).waitFor({ timeout: 60_000 });
await bob.getByText('2 comments').waitFor();
log('a new person voted, replied and voted the comment up');

await bob.goto(BASE);
await bob.getByRole('button', { name: 'Create', exact: true }).click();
await bob.getByRole('menuitem', { name: /Post/ }).click();
const body = `Planting garlic this weekend ${stamp}`;
await bob.getByLabel(/What's on your mind/).fill(body);
await bob.getByRole('button', { name: 'Publish post' }).click();
await bob.locator('.linked-body.detail', { hasText: body }).waitFor({ timeout: 120_000 });
assert.match(bob.url(), /\/posts\//);
const postComposer = bob.getByLabel('Write a comment...');
await postComposer.fill('Cloves pointy end up.');
await postComposer.press('Enter');
await bob.locator('.comment-card', { hasText: 'pointy end up' }).waitFor({ timeout: 60_000 });
log('they published a post and commented on it');

const profile = await bob.locator('.discussion-page .header-name').getAttribute('href');
assert.match(profile, /^\/profile\//);
for (let i = 0; ; i++) {
  await bob.goto(`${BASE}${profile}`);
  const card = bob.locator('.feed-stack .personal-post-card', { hasText: body });
  if (await card.waitFor({ timeout: 10_000 }).then(() => true, () => false)) break;
  if (i >= 20) throw new Error('the post never reached the profile');
  await bob.waitForTimeout(3000);
}
if (shots) await bob.screenshot({ path: `${shots}/profile-post.png` });
log('their profile shows it on the personal post card');

for (let i = 0; ; i++) {
  await ann.goto(url);
  const mine = ann.locator('.comment-card', { hasText: 'Tomatoes did best' }).first();
  await mine.waitFor({ timeout: 60_000 });
  const replied = await mine.locator('.reply-stack', { hasText: 'Which kind' }).count();
  const votes = (await mine.locator('.vote-count').first().textContent())?.trim();
  if (replied && votes === '1') break;
  if (i >= 20) throw new Error(`founder one never saw the reply and vote (votes ${votes})`);
  await ann.waitForTimeout(3000);
}
log('founder one sees the nested reply and the comment vote');

assert.deepEqual(errors, []);
await browser.close();
log('threads: ok');
