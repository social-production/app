import { entropyToMnemonic, mnemonicToEntropy, validateMnemonic } from '@scure/bip39';
import { wordlist } from '@scure/bip39/wordlists/english.js';

/**
 * Recovery phrase = 24 BIP-39 words encoding exactly the 32-byte seed Holochain imports.
 * The words ARE the seed (BIP-39 entropy, no password stretching), so the same phrase
 * restores the same agent key on any node or app (docs/IDENTITY.md).
 */
export function newPhrase(): string[] {
  const seed = crypto.getRandomValues(new Uint8Array(32));
  return entropyToMnemonic(seed, wordlist).split(' ');
}

export function phraseIsValid(words: string[]): boolean {
  return words.length === 24 && validateMnemonic(normalise(words), wordlist);
}

export function phraseToSeed(words: string[]): Uint8Array {
  if (!phraseIsValid(words)) throw new Error('That recovery phrase is not valid.');
  return mnemonicToEntropy(normalise(words), wordlist);
}

/** The first word that isn't in the word list, as "Word 7 'appel'" (for a helpful hint). */
export function firstInvalidWord(words: string[]): string | undefined {
  const index = words.findIndex((w) => !wordlist.includes(w.trim().toLowerCase()));
  return index < 0 ? undefined : `Word ${index + 1} “${words[index]}”`;
}

/** Three distinct random word positions (0-based) the person must re-enter. */
export function confirmationPositions(): number[] {
  const picks = new Set<number>();
  while (picks.size < 3) picks.add((crypto.getRandomValues(new Uint32Array(1))[0] ?? 0) % 24);
  return [...picks].sort((a, b) => a - b);
}

/** Split pasted text into words, tolerating numbering, commas and line breaks. */
export function splitWords(text: string): string[] {
  return text
    .toLowerCase()
    .split(/[^a-z]+/)
    .filter(Boolean);
}

function normalise(words: string[]): string {
  return words.map((w) => w.trim().toLowerCase()).join(' ');
}
