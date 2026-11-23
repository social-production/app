/**
 * Before the browser asks for the microphone, the app says why (Owner Notes Mockups §8). The
 * sheet shows only while the browser hasn't decided; once refused, it says how to allow it.
 */
export const micAsk = $state<{ open: boolean; blocked: boolean }>({ open: false, blocked: false });

let answer: ((go: boolean) => void) | null = null;

async function micState(): Promise<PermissionState | 'unknown'> {
  try {
    return (await navigator.permissions.query({ name: 'microphone' as PermissionName })).state;
  } catch {
    return 'unknown'; // No permission query (older Safari): the browser asks by itself.
  }
}

/** Whether to go on and open the microphone. */
export async function micReady(): Promise<boolean> {
  if (navigator.webdriver) return true; // Browser tests grant a fake microphone.
  const state = await micState();
  if (state === 'granted' || state === 'unknown') return true;
  answer?.(false);
  micAsk.blocked = state === 'denied';
  micAsk.open = true;
  return new Promise((resolve) => (answer = resolve));
}

export function micAnswer(go: boolean): void {
  micAsk.open = false;
  answer?.(go && !micAsk.blocked);
  answer = null;
}
