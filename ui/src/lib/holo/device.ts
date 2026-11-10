/**
 * The account on this device's own node, in the native app (ADR 0090). The window opens
 * unbound; these call the app's commands (native/src/account.rs), which install the hApp
 * for the phrase's agent and bind the window to it. The agent is the phrase's seed, as on
 * a gateway, so the same words are the same person everywhere (docs/IDENTITY.md).
 */

interface TauriWindow {
  __TAURI_INTERNALS__?: {
    invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  };
  __HC_TAURI_HOLOCHAIN__?: { INSTALLED_APP_ID?: string };
}

function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const internals = (window as TauriWindow).__TAURI_INTERNALS__;
  if (!internals) return Promise.reject(new Error('Not running in the app.'));
  return internals.invoke<T>(command, args);
}

/** Whether this device has an account (and, if so, the window is now bound to it). */
export async function deviceHasAccount(): Promise<boolean> {
  const has = await invoke<boolean>('account_state');
  if (has) await bound();
  return has;
}

/** Create (`restore` false) or log in to (`restore` true) the account of `seed`. A log-in
 * waits up to two minutes for the network to send the person's chain. */
export async function openOnDevice(seed: Uint8Array, restore: boolean): Promise<void> {
  await invoke<string>('open_account', { seed: Array.from(seed), restore });
  await bound();
}

/** Remove the account from this device; the words bring it back. */
export function signOutOnDevice(): Promise<void> {
  return invoke<null>('sign_out').then(() => undefined);
}

const BIND_WAIT_MS = 10_000;

/** The plugin tells the page about a rebind with an event that can arrive after the
 * command's reply, so wait for the injected app id before opening the client. */
function bound(): Promise<void> {
  const env = (window as TauriWindow).__HC_TAURI_HOLOCHAIN__;
  if (env?.INSTALLED_APP_ID) return Promise.resolve();
  return new Promise((resolve, reject) => {
    const done = (): void => {
      clearTimeout(timer);
      window.removeEventListener('holochain-rebound', done);
      if (env?.INSTALLED_APP_ID) resolve();
      else reject(new Error('The app did not open the account.'));
    };
    const timer = setTimeout(done, BIND_WAIT_MS);
    window.addEventListener('holochain-rebound', done);
  });
}
