/** Account flows for features (docs/IDENTITY.md): the recovery phrase always works;
 *  passkeys are an optional shortcut on devices that can store them. */
import { signUp, type AccountStep } from '#lib/holo/index.ts';

import { saveProfile } from './people.ts';

export {
  addPasskey,
  confirmationPositions,
  connect,
  devicePasskeysAvailable,
  firstInvalidWord,
  GatewayError,
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

/** A message fit to show a person, from any error an account flow throws. */
export function explain(error: unknown): string {
  if (error instanceof DOMException && error.name === 'NotAllowedError') {
    return 'The passkey step was cancelled or timed out.';
  }
  return error instanceof Error ? error.message : String(error);
}
