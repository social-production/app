import { generateSigningKeyPair } from '@holochain/client';

import { attach } from './connection.ts';
import { gatewayApi, type Ceremony, type GatewaySession, type SeedBody } from './gateway.ts';
import { saveSession, type DeviceSession, type KeyPair } from './keys.ts';
import { homeGateway } from './nodes.ts';
import { createPasskey, usePasskey } from './passkeys.ts';
import { phraseToSeed } from './phrase.ts';
import { solvePow } from './pow.ts';

/** Steps the UI can show while an account flow runs. */
export type AccountStep = 'checking' | 'passkey' | 'connecting';
type Progress = (step: AccountStep) => void;

/**
 * New account: the phrase's seed becomes the agent; a passkey binds this device.
 * `beforeShow` runs on the open connection before the app appears (the first profile).
 */
export async function signUp(
  name: string,
  words: string[],
  progress: Progress,
  beforeShow?: () => Promise<void>,
): Promise<void> {
  const gateway = requireGateway();
  const body = await seedBody(gateway, words, name, progress);
  const ceremony = await gatewayApi.signup(gateway, body);
  await register(gateway, ceremony, false, progress, beforeShow);
}

/** Existing account on a new device or node: same phrase, same agent, new passkey. */
export async function restore(words: string[], progress: Progress): Promise<void> {
  const gateway = requireGateway();
  const body = await seedBody(gateway, words, 'Social Production', progress);
  const ceremony = await gatewayApi.restore(gateway, body);
  await register(gateway, ceremony, true, progress);
}

export async function signIn(progress: Progress): Promise<void> {
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

async function seedBody(gateway: string, words: string[], name: string, progress: Progress) {
  const seed = phraseToSeed(words);
  progress('checking');
  const { challenge, bits } = await gatewayApi.pow(gateway);
  const nonce = await solvePow(challenge, bits);
  return { challenge, nonce, seed: Array.from(seed), name } satisfies SeedBody;
}

async function register(
  gateway: string,
  ceremony: Ceremony,
  restoring: boolean,
  progress: Progress,
  beforeShow?: () => Promise<void>,
) {
  progress('passkey');
  const credential = await createPasskey(ceremony.options);
  progress('connecting');
  const [keyPair, signingKey] = await generateSigningKeyPair();
  const session = await gatewayApi.register(gateway, {
    ceremony: ceremony.ceremony,
    credential,
    signingKey: Array.from(signingKey),
    restore: restoring,
  });
  await keep({ gateway, keyPair, signingKey, ...session }, beforeShow);
}

interface NewSession extends GatewaySession {
  gateway: string;
  keyPair: KeyPair;
  signingKey: Uint8Array;
}

/** Remember this device's session (for reloads) and connect with it. */
async function keep(input: NewSession, beforeShow?: () => Promise<void>): Promise<void> {
  const session: DeviceSession = {
    gateway: input.gateway,
    agent: input.agent,
    keyPair: input.keyPair,
    signingKey: input.signingKey,
    capSecret: new Uint8Array(input.capSecret),
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
