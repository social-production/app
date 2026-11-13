/** Account flows for features (docs/IDENTITY.md): the recovery phrase always works;
 *  passkeys are an optional shortcut on devices that can store them. */
import { callZome, peopleIndex, signUp, stringToHash, type AccountStep } from '#lib/holo/index.ts';

import { saveProfile, viewer } from './people.ts';

export {
  ACCOUNT_MOVED,
  addPasskey,
  confirmationPositions,
  connect,
  devicePasskeysAvailable,
  firstInvalidWord,
  GatewayError,
  inNativeApp,
  newPhrase,
  phraseIsValid,
  signInWithPasskey,
  signInWithPhrase,
  signOut,
  splitWords,
  type AccountStep,
} from '#lib/holo/index.ts';

/** Create the account and its first profile; the app appears once both exist. */
export async function createAccount(
  name: string,
  words: string[],
  progress: (step: AccountStep) => void,
): Promise<void> {
  await signUp(words, progress, () => firstProfile(name));
}

/** The new person's profile, written before the app appears; retried while the cell warms up. */
async function firstProfile(name: string): Promise<void> {
  const profile = { name, bio: '', home_area: '', avatar: null };
  for (let attempt = 1; ; attempt++) {
    try {
      await saveProfile(profile);
      return;
    } catch (error) {
      if (attempt >= 5) throw error;
      await new Promise((resolve) => setTimeout(resolve, attempt * 1000));
    }
  }
}

/**
 * Before a desktop signs out (ADR 0092): whether another node lists this account yet. The
 * device may be the only holder of a new chain, and signing out removes it. Today the gauge
 * is a gateway's people index; `undefined` when no gateway answers.
 */
export async function listedOnNetwork(): Promise<boolean | undefined> {
  try {
    // The name from this device's own chain: the page may not have loaded it yet.
    const me = await viewer();
    const people = await peopleIndex<{ agent: string }[]>({ q: me.name, limit: '50' });
    return people.some((person) => person.agent === me.agent);
  } catch {
    return undefined;
  }
}

/** web's deactivateAccount (ADR 0088): a `Deactivated` record, then the account's directory
 * chain closes for good. The caller signs out afterwards. */
export async function deactivateAccount(): Promise<void> {
  await callZome<number>('directory', 'identity', 'deactivate_account');
}

/** Whether `agent` deactivated its account (every client refuses to sign it in). */
export async function isDeactivated(agent: string): Promise<boolean> {
  const since = await callZome<number | null>(
    'directory',
    'identity',
    'deactivated_since',
    stringToHash(agent),
  );
  return since !== null;
}
