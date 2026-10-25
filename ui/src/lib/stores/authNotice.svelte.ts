import { connection } from './connection.svelte.ts';

/** web's authActionNotice: what a guest tried to do, with a link to sign in. */
export const authNotice = $state<{ message: string | null }>({ message: null });

/** web's requireViewer: true when signed in; otherwise shows the notice and returns false. */
export function requireViewer(message?: string): boolean {
  if (connection.current.status === 'connected') return true;
  authNotice.message = message?.trim() || 'Sign in to continue.';
  return false;
}

export function dismissAuthNotice(): void {
  authNotice.message = null;
}
