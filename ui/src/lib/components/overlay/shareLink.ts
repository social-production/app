import { getContext, setContext } from 'svelte';

/**
 * Share on cards and comments opens the app's one share sheet (Owner Notes Mockups §6).
 * Components stay presentational (CODE_RULES): the shell provides it.
 */
export type ShareKind =
  'project' | 'event' | 'help request' | 'thread' | 'post' | 'comment' | 'profile';

export interface ShareSubject {
  kind: ShareKind;
  title: string;
  /** Resolved path (+ query) of what is shared; the sheet adds the origin. */
  path: string;
}

export interface ShareLink {
  open: (subject: ShareSubject) => void;
}

const KEY = Symbol('share-link');

export function provideShareLink(share: ShareLink): void {
  setContext(KEY, share);
}

/** None outside the shell: callers fall back to the device's share sheet. */
export function shareLink(): ShareLink | undefined {
  return getContext<ShareLink | undefined>(KEY);
}
