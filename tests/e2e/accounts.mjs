// M1 accounts end to end, in Chromium, against a local seed node. Two kinds of device:
// one with no passkey store (like many Linux desktops) and one with a virtual passkey.
// Run `bash scripts/dev-seed.sh` (RP_ORIGIN must match the UI origin), serve the UI built
// with VITE_SP_GATEWAYS=http://localhost:8080, then:
//   node tests/e2e/accounts.mjs http://localhost:4174
import assert from 'node:assert/strict';
import { deflateSync } from 'node:zlib';

import { BASE, browser, device, onboarding, errors, keyOf, log, open, phraseSignIn, signedIn, signOut, signUp } from './lib.mjs';

// 1. A device with no passkey store: everything works with the phrase, no passkey prompts.
const a = await device({ passkeys: false });
await onboarding(a);
await a.getByRole('tab', { name: 'Log in' }).click();
assert.equal(await a.getByRole('button', { name: 'Log in with a passkey' }).count(), 0);
const t0 = Date.now();
const words = await signUp(a, 'AnnTester');
log(`signed up in ${((Date.now() - t0) / 1000).toFixed(1)}s`);
assert.equal(await a.getByText('Save a passkey').count(), 0, 'no passkey offer without a store');
const annKey = await keyOf(a);

await open(a, 'Settings');
// Each profile field is a row that opens a small sheet (ADR 0105).
await a.getByRole('button', { name: /^Home area:/ }).click();
await a.getByRole('textbox', { name: 'Home area' }).fill('Brisbane, QLD');
await a.getByRole('button', { name: 'Save profile' }).click();
await a.getByText('Saved.').waitFor({ timeout: 60_000 });
await a.getByRole('button', { name: 'Home area: Brisbane, QLD' }).waitFor();
// A profile photo (a 4×4 PNG made here); the app resizes it and saves it at once.
await a.locator('#settings-profile input[type=file]').setInputFiles({
  name: 'me.png',
  mimeType: 'image/png',
  buffer: tinyPng(),
});
await a.getByText(/Picture resized.*Saved\./).waitFor({ timeout: 60_000 });
await a.reload();
await signedIn(a);
log('profile saved; reload kept the session');

// web's ProfilePage: the photo opens full screen (PhotoViewer); Escape closes it.
await a.goto(`${BASE}/profile`);
await a.getByRole('button', { name: 'View AnnTester profile photo' }).click({ timeout: 60_000 });
await a.getByRole('dialog', { name: 'AnnTester profile photo' }).locator('img').waitFor();
await a.keyboard.press('Escape');
await a.getByRole('dialog', { name: 'AnnTester profile photo' }).waitFor({ state: 'detached' });
log('the profile photo opens in the photo viewer');

await signOut(a);
await onboarding(a);
await a.getByRole('tab', { name: 'Log in' }).click();
await a.getByLabel('Recovery phrase').fill([...words.slice(0, 6), 'appel', ...words.slice(7)].join(' '));
await a.getByText(/Word 7 .appel. isn't in the word list/).waitFor({ timeout: 5000 });
log('typo in the phrase is named');
await phraseSignIn(a, words);
assert.equal(await keyOf(a), annKey);
log('signed back in with the phrase (no passkey)');

// 2. A device with a passkey store: phrase sign-in, save a passkey, then use it.
const b = await device({ passkeys: true });
await phraseSignIn(b, words);
assert.equal(await keyOf(b), annKey, 'same identity on a second device');
await b.goto(BASE);
await signedIn(b);
await b.getByRole('button', { name: 'Save a passkey' }).click();
await b.getByText('Sign in faster on this device').waitFor({ state: 'detached', timeout: 60_000 });
await signOut(b);
await onboarding(b);
await b.getByRole('tab', { name: 'Log in' }).click();
await b.getByRole('button', { name: 'Log in with a passkey' }).click();
await signedIn(b);
assert.equal(await keyOf(b), annKey);
log('saved a passkey and signed in with it');

// 3. Pairing: another person's device asks to pair with Ann; Ann confirms.
const c = await device({ passkeys: false });
await signUp(c, 'AnnLaptop');
await open(c, 'Settings');
await c.getByPlaceholder('uhCAk…').fill(annKey);
await c.getByRole('button', { name: 'Send pairing request' }).click();
await c.getByText('Request sent').waitFor({ timeout: 60_000 });
await open(a, 'Settings');
for (let i = 0; i < 20 && !(await a.getByRole('button', { name: 'Confirm' }).count()); i++) {
  await a.reload();
  await signedIn(a);
  await open(a, 'Settings');
  await a.waitForTimeout(3000);
}
await a.getByRole('button', { name: 'Confirm' }).click();
await a.locator('#settings-devices', { hasText: 'AnnLaptop' }).waitFor({ timeout: 60_000 });
await a.goto(`${BASE}/people`);
await a.getByText('2 devices').first().waitFor({ timeout: 60_000 });
assert.equal(await a.locator('.person-row', { hasText: 'AnnLaptop' }).count(), 0, 'paired device folded into its person');
log('devices paired and shown as one person');

assert.deepEqual(errors, [], 'no page errors');
log('all M1 account checks passed');
await browser.close();

/** A 4×4 green PNG, built by hand (no image files in the repo's tests). */
function tinyPng() {
  const crcTable = Array.from({ length: 256 }, (_, n) => {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    return c >>> 0;
  });
  const crc = (buf) => {
    let c = 0xffffffff;
    for (const b of buf) c = crcTable[(c ^ b) & 0xff] ^ (c >>> 8);
    return (c ^ 0xffffffff) >>> 0;
  };
  const chunk = (type, data) => {
    const len = Buffer.alloc(4);
    len.writeUInt32BE(data.length);
    const body = Buffer.concat([Buffer.from(type), data]);
    const sum = Buffer.alloc(4);
    sum.writeUInt32BE(crc(body));
    return Buffer.concat([len, body, sum]);
  };
  const header = Buffer.alloc(13);
  header.writeUInt32BE(4, 0);
  header.writeUInt32BE(4, 4);
  header.set([8, 2, 0, 0, 0], 8);
  const row = Buffer.from([0, ...Array(4).fill([46, 139, 87]).flat()]);
  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', header),
    chunk('IDAT', deflateSync(Buffer.concat([row, row, row, row]))),
    chunk('IEND', Buffer.alloc(0)),
  ]);
}
