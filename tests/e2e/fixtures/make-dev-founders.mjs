// Prints two fixed test identities (phrase + agent key) for the local dev hApp's founders.
// They are public test data: never founders of a real network (scripts/dev-seed.sh only).
//   node tests/e2e/fixtures/make-dev-founders.mjs > tests/e2e/fixtures/dev-founders.json
import { createHash } from 'node:crypto';

import { encodeHashToBase64, HoloHashType, hashFrom32AndType } from '@holochain/client';
import { entropyToMnemonic } from '@scure/bip39';
import { wordlist } from '@scure/bip39/wordlists/english.js';
import sodium from 'libsodium-wrappers';

await sodium.ready;
const founders = ['Dev Founder One', 'Dev Founder Two'].map((name, i) => {
  const seed = createHash('sha256').update(`sp-dev-founder-${i + 1}`).digest();
  const { publicKey } = sodium.crypto_sign_seed_keypair(seed);
  const agent = encodeHashToBase64(hashFrom32AndType(publicKey, HoloHashType.Agent));
  return { name, agent, phrase: entropyToMnemonic(seed, wordlist) };
});
console.log(JSON.stringify(founders, null, 2));
