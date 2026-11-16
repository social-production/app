// Web Push in a browser (ADR 0107): the service worker, the browser's permission and its push
// subscription, saved on the gateway that hosts this account. The desktop app shows its own
// notifications instead, so none of this runs there.
import { gatewayApi } from './gateway.ts';
import { loadSession } from './keys.ts';
import { homeGateway, inNativeApp } from './nodes.ts';

export interface PushChoices {
  /** Notification categories switched off on this device. */
  muted: string[];
  /** "Show what it's about": off, a push only says something is new. */
  details: boolean;
}

/** Whether this browser can get pushes here (iPhones: only from the Home Screen). */
export function pushSupported(): boolean {
  return (
    !inNativeApp() &&
    typeof window !== 'undefined' &&
    'serviceWorker' in navigator &&
    'PushManager' in window &&
    'Notification' in window
  );
}

/** Whether pushes are on in this browser. */
export async function pushIsOn(): Promise<boolean> {
  if (!pushSupported() || Notification.permission !== 'granted') return false;
  const registration = await navigator.serviceWorker.getRegistration('/');
  return Boolean(await registration?.pushManager.getSubscription());
}

/** Ask for permission, subscribe and save the subscription on the gateway. False if refused. */
export async function turnPushOn(choices: PushChoices): Promise<boolean> {
  const { gateway, device } = await signedIn();
  if (!pushSupported()) return false;
  if ((await Notification.requestPermission()) !== 'granted') return false;
  const registration = await navigator.serviceWorker.register('/sw.js', { scope: '/' });
  await navigator.serviceWorker.ready;
  const { publicKey } = await gatewayApi.pushKey(gateway);
  const existing = await registration.pushManager.getSubscription();
  const subscription =
    existing ??
    (await registration.pushManager.subscribe({
      userVisibleOnly: true,
      applicationServerKey: fromBase64Url(publicKey),
    }));
  const json = subscription.toJSON() as {
    endpoint: string;
    keys: { p256dh: string; auth: string };
  };
  await gatewayApi.pushSubscribe(gateway, { ...device, subscription: json, ...choices });
  return true;
}

/** Save new choices for the subscription this browser already has. */
export async function updatePushChoices(choices: PushChoices): Promise<void> {
  if (await pushIsOn()) await turnPushOn(choices);
}

/** Stop pushes in this browser (Settings, and signing out). */
export async function turnPushOff(): Promise<void> {
  if (!pushSupported()) return;
  const registration = await navigator.serviceWorker.getRegistration('/');
  const subscription = await registration?.pushManager.getSubscription();
  if (!subscription) return;
  const { endpoint } = subscription;
  await subscription.unsubscribe();
  const { gateway, device } = await signedIn().catch(() => ({ gateway: '', device: null }));
  if (gateway && device) await gatewayApi.pushUnsubscribe(gateway, { ...device, endpoint });
}

async function signedIn() {
  const gateway = homeGateway();
  const session = gateway ? await loadSession(gateway) : undefined;
  if (!gateway || !session) throw new Error('Please sign in again first.');
  const device = {
    signingKey: Array.from(session.signingKey),
    deviceSecret: Array.from(session.deviceSecret),
  };
  return { gateway, device };
}

function fromBase64Url(text: string): Uint8Array<ArrayBuffer> {
  const base64 = text.replace(/-/g, '+').replace(/_/g, '/');
  const raw = atob(base64 + '='.repeat((4 - (base64.length % 4)) % 4));
  return Uint8Array.from(raw, (c) => c.charCodeAt(0));
}
