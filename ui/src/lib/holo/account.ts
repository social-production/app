import { generateSigningKeyPair } from '@holochain/client';

import { attach } from './connection.ts';
import { gatewayApi, GatewayError, type GatewaySession, type OpenMode } from './gateway.ts';
import { loadSession, saveSession, type DeviceSession, type KeyPair } from './keys.ts';
import { homeGateway } from './nodes.ts';
import { createPasskey, usePasskey } from './passkeys.ts';
import { phraseToSeed } from './phrase.ts';
import { solvePow } from './pow.ts';

/** Steps the UI can show while an account flow runs. */
export type AccountStep = 'checking' | 'passkey' | 'connecting';
type Progress = (step: AccountStep) => void;
type BeforeShow = () => Promise<void>;

/** New account from a fresh phrase. `beforeShow` runs before the app appears. */
export function signUp(words: string[], progress: Progress, beforeShow?: BeforeShow) {
  return openWithPhrase('signup', words, progress, beforeShow);
}

/**
 * Sign in with the recovery phrase. If this site doesn't host the account yet (it was made
 * on another node or the app), it is restored here: same identity, new home.
 */
export async function signInWithPhrase(words: string[], progress: Progress): Promise<void> {
  try {
    await openWithPhrase('signin', words, progress);
  } catch (error) {
    if (!(error instanceof GatewayError && error.status === 404)) throw error;
    await openWithPhrase('restore', words, progress);
  }
}

export async function signInWithPasskey(progress: Progress): Promise<void> {
  const gateway = requireGateway();
  const ceremony = await gatewayApi.loginStart(gateway);
  progress('passkey');
  const credential = await usePasskey(ceremony.options);
  progress('connecting');
  const [keyPair, signingKey] = await generateSigningKeyPair();
  const session = await gatewayApi.loginFinish(gateway, {
    ceremony: ceremony.ceremony,
    credential,
    signingKey: Array.from(signingKey),
  });
  await keep({ gateway, keyPair, signingKey, ...session });
}

/** Save a passkey on this (signed-in) device so next time no words are needed. */
export async function addPasskey(name: string): Promise<void> {
  const gateway = requireGateway();
  const session = await loadSession(gateway);
  if (!session) throw new Error('Please sign in again first.');
  const ceremony = await gatewayApi.passkeyStart(gateway, {
    signingKey: Array.from(session.signingKey),
    deviceSecret: Array.from(session.deviceSecret),
    name,
  });
  const credential = await createPasskey(ceremony.options);
  await gatewayApi.passkeyFinish(gateway, { ceremony: ceremony.ceremony, credential });
}

async function openWithPhrase(
  mode: OpenMode,
  words: string[],
  progress: Progress,
  beforeShow?: BeforeShow,
): Promise<void> {
  const gateway = requireGateway();
  const seed = Array.from(phraseToSeed(words));
  let proof: { challenge?: string; nonce?: string } = {};
  if (mode !== 'signin') {
    progress('checking');
    const { challenge, bits } = await gatewayApi.pow(gateway);
    proof = { challenge, nonce: await solvePow(challenge, bits) };
  }
  progress('connecting');
  const [keyPair, signingKey] = await generateSigningKeyPair();
  const session = await gatewayApi.open(gateway, {
    mode,
    ...proof,
    seed,
    signingKey: Array.from(signingKey),
  });
  await keep({ gateway, keyPair, signingKey, ...session }, beforeShow);
}

interface NewSession extends GatewaySession {
  gateway: string;
  keyPair: KeyPair;
  signingKey: Uint8Array;
}

/** Remember this device's session (for reloads) and connect with it. */
async function keep(input: NewSession, beforeShow?: BeforeShow): Promise<void> {
  const session: DeviceSession = {
    gateway: input.gateway,
    agent: input.agent,
    keyPair: input.keyPair,
    signingKey: input.signingKey,
    capSecret: new Uint8Array(input.capSecret),
    deviceSecret: new Uint8Array(input.deviceSecret),
    cells: input.cells,
  };
  await saveSession(session);
  await attach(session, input.appWsUrl, input.token, beforeShow);
}

function requireGateway(): string {
  const gateway = homeGateway();
  if (!gateway) throw new Error('No node configured (VITE_SP_GATEWAYS).');
  return gateway;
}
