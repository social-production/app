// Voice calls in a direct chat at phone width (ADR 0050, web's CallHost). Two new accounts:
// Ann messages Bob, then calls him from the chat header. Bob's screen rings wherever he is in
// the app; he declines the first call (Ann sees "Call declined"), accepts the second (both
// see the timer, audio from Chromium's fake microphone), and Ann ends it. With Bob's app
// closed, Ann's next call says "Not available right now". Each call leaves web's line in the
// chat (ADR 0051): "Call declined", one "Voice call · 0:0X" on both phones, then "No answer".
//   node tests/e2e/calls.mjs http://localhost:4174 [screens dir]
// It signs up two test accounts, so it may run against the live test network too.
import assert from 'node:assert/strict';
import { mkdirSync } from 'node:fs';

import { BASE, browser, device, errors, log, signUp } from './lib.mjs';

const out = process.argv[3] ?? 'screens';
mkdirSync(out, { recursive: true });
const phone = { width: 390, height: 844 };
const stamp = Date.now().toString(36).slice(-5);
const [annName, bobName] = [`ann${stamp}`, `bob${stamp}`];
const hello = `Can I call you? ${stamp}`;

const screen = (page) => page.locator('.call-screen');
const status = (page) => page.locator('.call-screen .call-status');
const line = (page, text) => page.locator('.chat-panel .call-line', { hasText: text });
const row = (page, name) => page.locator('.conversation-row', { hasText: name }).first();

async function statusIs(page, pattern, timeout = 60_000) {
  await page.locator('.call-screen .call-status', { hasText: pattern }).waitFor({ timeout });
}

const ann = await device({ viewport: phone });
await signUp(ann, annName);
const bob = await device({ viewport: phone });
await signUp(bob, bobName);
await bob.goto(`${BASE}/settings`);
const key = bob.locator('#settings-account .key');
await key.waitFor({ timeout: 60_000 });
const bobKey = (await key.innerText()).trim();
log('two accounts signed up');

await ann.goto(`${BASE}/profile/${bobKey}`);
await ann.getByRole('button', { name: `Message ${bobName}` }).click({ timeout: 120_000 });
await ann.getByPlaceholder('Write a message…').fill(hello);
await ann.getByRole('dialog').getByRole('button', { name: 'Send' }).click();
await ann.waitForURL(/\/messages\?conversation=/, { timeout: 120_000 });
const chatUrl = ann.url();
log('ann messaged bob');

// Bob's node learns of the chat through his inbox; wait until his list shows it.
for (let i = 0; i < 20 && !(await row(bob, annName).count()); i++) {
  await bob.goto(`${BASE}/messages`);
  await bob.waitForTimeout(5000);
}
await row(bob, annName).waitFor({ timeout: 10_000 });
await bob.goto(`${BASE}/`);
log('bob has the chat');

const callButton = ann.getByRole('button', { name: `Call ${bobName}` });
await ann.screenshot({ path: `${out}/phone390-call-button.png` });
await callButton.click();
await statusIs(ann, 'Ringing…', 10_000);
await statusIs(bob, 'Incoming voice call');
assert.equal(await bob.locator('#call-title').innerText(), annName, 'the ring names Ann');
await bob.screenshot({ path: `${out}/phone390-call-ringing.png` });
await bob.getByRole('button', { name: 'Decline' }).click();
await statusIs(ann, 'Call declined');
log('bob declined the first call');

await screen(ann).waitFor({ state: 'detached', timeout: 15_000 });
await callButton.click();
await statusIs(bob, 'Incoming voice call');
await bob.getByRole('button', { name: 'Accept' }).click();
await statusIs(ann, /^\d\d:\d\d$/);
await statusIs(bob, /^\d\d:\d\d$/);
await ann.waitForTimeout(3000);
assert.match(await status(ann).innerText(), /^00:0[2-9]$/, 'the timer runs');
await ann.screenshot({ path: `${out}/phone390-call-active.png` });
await ann.getByRole('button', { name: 'Back to the app' }).click();
await ann.locator('.call-pill').waitFor();
await ann.screenshot({ path: `${out}/phone390-call-pill.png` });
await ann.locator('.call-pill').click();
await ann.getByRole('button', { name: 'End call' }).click();
await statusIs(bob, /^Call ended/);
log('bob accepted the second call; ann ended it');

await line(ann, 'Call declined').waitFor({ timeout: 60_000 });
await line(ann, /Voice call · 0:0\d/).waitFor({ timeout: 60_000 });
await ann.screenshot({ path: `${out}/phone390-call-lines.png` });
await bob.goto(chatUrl);
await line(bob, /Voice call · 0:0\d/).waitFor({ timeout: 120_000 });
await line(bob, 'Call declined').waitFor({ timeout: 60_000 });
await bob.waitForTimeout(6000);
assert.equal(await line(bob, 'Voice call').count(), 1, 'both phones logged it: one line');
assert.equal(await line(ann, 'Voice call').count(), 1, 'one line for Ann too');
await bob.goto(`${BASE}/`);
log('the declined and the finished call left their lines');

await bob.context().close();
await screen(ann).waitFor({ state: 'detached', timeout: 15_000 });
await ann.goto(chatUrl);
await callButton.click();
await statusIs(ann, 'Not available right now', 40_000);
log('a call to a closed app is not available');
await line(ann, 'No answer').waitFor({ timeout: 60_000 });
log('it left "No answer"');

assert.deepEqual(errors, [], 'no page errors');
log('calls ok');
await browser.close();
