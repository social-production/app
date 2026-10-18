import { sha256 } from '@noble/hashes/sha2.js';

const encoder = new TextEncoder();

/**
 * Find a nonce so sha256("<challenge>:<nonce>") has `bits` leading zero bits (the
 * gateway's signup check). Yields to the page every few thousand tries so it stays usable.
 */
export async function solvePow(
  challenge: string,
  bits: number,
  onProgress?: (tries: number) => void,
): Promise<string> {
  for (let nonce = 0; ; nonce++) {
    if (leadingZeroBits(sha256(encoder.encode(`${challenge}:${nonce}`))) >= bits) {
      return String(nonce);
    }
    if (nonce % 5000 === 0) {
      onProgress?.(nonce);
      await new Promise((resolve) => setTimeout(resolve, 0));
    }
  }
}

function leadingZeroBits(hash: Uint8Array): number {
  let bits = 0;
  for (const byte of hash) {
    if (byte === 0) {
      bits += 8;
      continue;
    }
    return bits + Math.clz32(byte) - 24;
  }
  return bits;
}
