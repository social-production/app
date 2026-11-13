import { isDeactivated, signOut } from '#lib/services/account.ts';
import { DEACTIVATED } from '#lib/services/errors.ts';
import { authNotice } from '#lib/stores/authNotice.svelte.ts';

/** Signing out reloads the page, so the reason waits in session storage until it shows. */
const KEY = 'sp.deactivated';
const MESSAGE = DEACTIVATED;

/** ADR 0088: every client refuses a deactivated account, not a server. */
export async function refuseIfDeactivated(agent: string): Promise<void> {
  if (!(await isDeactivated(agent).catch(() => false))) return;
  try {
    sessionStorage.setItem(KEY, '1');
  } catch {
    // Private mode: sign out without the reason.
  }
  await signOut();
}

/** After the reload: say why the account was signed out. */
export function showDeactivatedReason(): void {
  try {
    if (sessionStorage.getItem(KEY) !== '1') return;
    sessionStorage.removeItem(KEY);
    authNotice.message = MESSAGE;
  } catch {
    // No session storage: nothing to show.
  }
}
