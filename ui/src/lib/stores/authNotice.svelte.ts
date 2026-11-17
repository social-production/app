import { connection } from './connection.svelte.ts';

/** web's authActionNotice: a note with a link to sign in (now only why an account was
 * signed out); `signIn` asks the shell to open the sign-up page. */
export const authNotice = $state<{ message: string | null; signIn: number }>({
  message: null,
  signIn: 0,
});

/** web's requireViewer: true when signed in. A guest goes straight to the sign-up page
 * (owner, 2026-11-17: no small "Sign in to …" pop-ups); `message` is kept for callers. */
export function requireViewer(message?: string): boolean {
  void message;
  if (connection.current.status === 'connected') return true;
  if (connection.current.status === 'signed-out') authNotice.signIn++;
  return false;
}

export function dismissAuthNotice(): void {
  authNotice.message = null;
}
