import { decodeHashFromBase64, encodeHashToBase64 } from '@holochain/client';

import { callZome, connection, onNoticeSignal } from './connection.ts';

/**
 * Notices (ADR 0108): a write that makes a card for someone also tells them at once, through
 * the directory's `notices` zome (a remote signal, never stored). The list itself is still
 * built from the chains, so a lost notice costs nothing.
 */
export interface Notice {
  kind: string;
  category: string;
  title: string;
  body: string;
  /** An in-app path; opening the notification goes there. */
  link: string;
}

const ENABLED = 'sp.notices.enabled.';

/** Let other people's apps reach this one. Once per account per device (a repeat is a no-op
 * on the chain anyway). */
export async function enableNotices(): Promise<void> {
  const { ws } = await connection();
  const key = ENABLED + encodeHashToBase64(ws.myPubKey);
  if (stored(key)) return;
  await callZome<boolean>('directory', 'notices', 'enable_notices', null);
  try {
    localStorage.setItem(key, '1');
  } catch {
    /* Private windows may refuse storage; the call is simply repeated next time. */
  }
}

function stored(key: string): boolean {
  try {
    return localStorage.getItem(key) === '1';
  } catch {
    return false;
  }
}

/** Tell `to` (agent keys) about something new. Best effort: a failure changes nothing. */
export async function sendNotice(to: string[], notice: Notice): Promise<void> {
  if (!to.length) return;
  await callZome<number>('directory', 'notices', 'send_notice', {
    to: to.map((agent) => decodeHashFromBase64(agent)),
    notice,
  });
}

function isNotice(value: unknown): value is Notice {
  if (!value || typeof value !== 'object') return false;
  const n = value as Record<string, unknown>;
  return ['kind', 'category', 'title', 'body', 'link'].every((k) => typeof n[k] === 'string');
}

/** Notices as they arrive while the app is open. */
export function onNotice(listener: (notice: Notice) => void): () => void {
  return onNoticeSignal((payload) => {
    const notice = (payload as { notice?: unknown } | null)?.notice;
    if (isNotice(notice)) listener(notice);
  });
}
