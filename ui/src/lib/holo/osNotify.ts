/**
 * The desktop app's own notifications (ADR 0108): the OS shows them through Tauri's
 * notification plugin (native/Cargo.toml). Nothing goes through a seed or a push service.
 */

interface TauriWindow {
  __TAURI_INTERNALS__?: {
    invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  };
}

function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const internals = (window as TauriWindow).__TAURI_INTERNALS__;
  if (!internals) return Promise.reject(new Error('Not running in the app.'));
  return internals.invoke<T>(command, args);
}

/** Whether the OS lets the app show notifications, asking once if it hasn't been asked. */
export async function osNotificationsAllowed(): Promise<boolean> {
  if (await invoke<boolean | null>('plugin:notification|is_permission_granted')) return true;
  const answer = await invoke<string>('plugin:notification|request_permission');
  return answer === 'granted';
}

/** Show one notification. Best effort: a refusal changes nothing in the app. */
export async function osNotify(title: string, body: string): Promise<void> {
  await invoke('plugin:notification|notify', { options: { title, body } });
}
