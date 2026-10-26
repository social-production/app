// Guests open project pages signed out, as on web (STATUS Next 3, ADR 0052). Ann starts a
// project and writes in its chat; a signed-out phone opens the page from the feed, reads
// Details, Take part and Chat, and gets web's sign-in notice from Join and from the composer.
//   node tests/e2e/guests.mjs http://localhost:4174 [screens dir]
// It signs up one test account, so it may run against the live test network too.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, device, errors, log, signUp } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const title = `Seed swap ${stamp}`;
const hello = `Bring your spare seeds ${stamp}`;

const ann = await device({ viewport: phone });
await signUp(ann, `ann${stamp}`);
await ann.goto(`${BASE}/create/project`);
await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByLabel('Title').fill(title);
await ann.getByLabel('Proposal description').fill('Swap seeds every spring.');
for (let i = 0; i < 3; i++) await ann.getByRole('button', { name: 'Continue' }).click();
await ann.getByRole('button', { name: 'Publish project' }).click();
await ann.getByRole('heading', { name: title }).waitFor({ timeout: 120_000 });
const projectUrl = ann.url();
await ann.getByRole('tab', { name: 'Chat' }).click();
await ann.getByRole('textbox', { name: 'Message the project...' }).fill(hello);
await ann.getByRole('button', { name: 'Send', exact: true }).click();
await ann.locator('.chat-log').getByText('Sending…').waitFor({ state: 'detached', timeout: 60_000 });
log('ann created the project and wrote in its chat');

// The guest finds it in the feed (the archivist joins new projects within a round).
const guest = await device({ viewport: phone });
const card = guest.locator('.project-card', { hasText: title }).first();
for (let i = 0; i < 20 && !(await card.count()); i++) {
  await guest.waitForTimeout(3000);
  await guest.reload();
}
// web's comments pill: Ann's message counts (the index refreshes every archivist round).
const pill = card.getByLabel('1 comment');
for (let i = 0; i < 20 && !(await pill.count()); i++) {
  await guest.waitForTimeout(3000);
  await guest.reload();
}
await pill.waitFor();
// web's profile feed: the author link opens Ann's profile, which lists her project.
await card.locator('.content-meta-row a').first().click();
await guest.waitForURL(/\/profile\//);
await guest.locator('.feed-stack .personal-activity-card', { hasText: title }).waitFor({ timeout: 60_000 });
await guest.screenshot({ path: `${out}/phone390-guest-profile-feed.png` });
const authors = await guest.locator('.feed-stack .identity .header-name').allInnerTexts();
assert.deepEqual([...new Set(authors)], [`ann${stamp}`], 'only her projects');
await guest.goBack();
log('the guest saw the project on its author\'s profile');
await card.locator('a.title').click();
await guest.getByRole('heading', { name: title }).waitFor({ timeout: 60_000 });
assert.equal(new URL(guest.url()).pathname, new URL(projectUrl).pathname);
assert.equal(await guest.locator('.shell[data-signed-in="false"]').count(), 1);
await guest.getByText('Swap seeds every spring.').first().waitFor();
await guest.screenshot({ path: `${out}/phone390-guest-details.png` });
log('the guest opened the project page signed out');

// web's requireViewer: Join shows "Sign in to join this project." with a Sign in link.
await guest.getByRole('button', { name: 'Join project' }).first().click();
const notice = guest.locator('.auth-action-notice');
await notice.getByText('Sign in to join this project.').waitFor();
assert.equal(await notice.getByRole('link', { name: 'Sign in' }).getAttribute('href'), '/onboarding');
await guest.screenshot({ path: `${out}/phone390-guest-notice.png` });
await notice.getByRole('button', { name: 'Dismiss' }).click();
await notice.waitFor({ state: 'detached' });

await guest.getByRole('tab', { name: /Take part|Participation/ }).click();
await guest.locator('.participation-tab').getByText(/Proposal/).first().waitFor();

await guest.getByRole('tab', { name: 'Chat' }).click();
await guest.locator('.chat-log').getByText(hello).waitFor({ timeout: 60_000 });
await guest.getByRole('button', { name: 'Sign in', exact: true }).click();
await notice.getByText('Sign in to send a message.').waitFor();
await guest.screenshot({ path: `${out}/phone390-guest-chat.png` });
log('the guest read the chat; sending asks them to sign in');

assert.deepEqual(errors, [], 'no page errors');
log('all guest checks passed');
await browser.close();
