import { isDeactivated, signOut } from '#lib/services/account.ts';
import { authNotice } from '#lib/stores/authNotice.svelte.ts';

/** ADR 0088: every client refuses a deactivated account, not a server. */
export async function refuseIfDeactivated(agent: string): Promise<void> {
  if (!(await isDeactivated(agent).catch(() => false))) return;
  await signOut();
  authNotice.message = 'This account was deactivated, so it can no longer sign in.';
}
