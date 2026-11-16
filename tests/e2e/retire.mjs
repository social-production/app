// ADR 0092: a hosted account moves to the desktop app. The desktop signs a retirement with
// the phrase's key (here, Node's crypto does what native/src/retire.rs does); the gateway
// stops hosting the account, refuses to sign it in, and hosts it again only when the person
// chooses "Use it here instead", restoring the chain from the network.
//   CHROMIUM_PATH=/opt/pw-browsers/chromium bash scripts/e2e-local.sh tests/e2e/retire.mjs
import assert from 'node:assert/strict';
import { createPrivateKey, sign } from 'node:crypto';

import { generateMnemonic, mnemonicToEntropy } from '@scure/bip39';
import { wordlist } from '@scure/bip39/wordlists/english.js';

import { BASE, browser, device, errors, keyOf, log, onboarding, signedIn, signUp } from './lib.mjs';

const GATEWAY = process.env.SP_GATEWAY ?? 'http://localhost:8080';

/** What the desktop sends: the statement signed by the phrase's ed25519 key. */
function retirement(words, agent, at = Math.floor(Date.now() / 1000)) {
  const seed = Buffer.from(mnemonicToEntropy(words.join(' '), wordlist));
  const der = Buffer.concat([Buffer.from('302e020100300506032b657004220420', 'hex'), seed]);
  const key = createPrivateKey({ key: der, format: 'der', type: 'pkcs8' });
  const statement = `social-production: retire the hosted copy of ${agent} at ${at}`;
  return { agent, at, signature: [...sign(null, Buffer.from(statement), key)] };
}

async function retire(body) {
  const response = await fetch(`${GATEWAY}/account/retire`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
  return { status: response.status, json: await response.json() };
}

const a = await device();
const name = `Mover${Date.now() % 100_000}`;
const words = await signUp(a, name);
const agent = await keyOf(a);
log(`signed up ${name} on the gateway`);

// Someone else's words can't retire the account, nor can an old statement.
const stranger = retirement(generateMnemonic(wordlist, 256).split(' '), agent);
assert.equal((await retire(stranger)).status, 401);
assert.equal((await retire(retirement(words, agent, 1_700_000_000))).status, 400);
log('a wrong key and a stale statement are refused');

const moved = await retire(retirement(words, agent));
assert.equal(moved.status, 200);
assert.equal(moved.json.retired, true);
log('the desktop retired the hosted copy');

// The browser's session is gone; logging in names the move and offers to take it back.
await a.reload();
await a.locator('.shell[data-signed-in="false"]').waitFor({ timeout: 60_000 });
await onboarding(a);
await a.getByRole('tab', { name: 'Log in' }).click();
await a.getByLabel('Recovery phrase').fill(words.join(' '));
await a.getByRole('button', { name: 'Log in', exact: true }).click();
await a.getByText('This account moved to the desktop app.').waitFor({ timeout: 60_000 });
log('log-in says the account moved');

await a.getByRole('button', { name: 'Use it here instead' }).click();
await signedIn(a);
assert.equal(await keyOf(a), agent);
await a.goto(`${BASE}/profile`);
await a.getByText(name).first().waitFor({ timeout: 120_000 });
log('"Use it here instead" hosts it again with its profile');

assert.deepEqual(errors, []);
log('PASS');
await browser.close();
