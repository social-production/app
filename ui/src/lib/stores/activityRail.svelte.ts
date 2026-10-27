/**
 * The right rail's cards and the ones you cleared or opened (web's `dismissedRailItems.ts`).
 * Cleared and seen ids stay in the browser under web's keys, per person; the shell fills
 * `items` and `history` while someone is signed in.
 */
import type { RailItem } from '#lib/domain/activityRail.ts';

export const activityRail = $state<{
  items: RailItem[];
  history: RailItem[];
  loaded: boolean;
  viewer: string | null;
  /** Ids as lists, so a change is a new value the shell's badge sees. */
  dismissed: string[];
  seen: string[];
}>({ items: [], history: [], loaded: false, viewer: null, dismissed: [], seen: [] });

const dismissedKey = (viewer: string | null): string =>
  viewer ? `dismissed-rail-ids-${viewer}` : 'dismissed-rail-ids';
const seenKey = (viewer: string | null): string =>
  viewer ? `seen-rail-ids-${viewer}` : 'seen-rail-ids';

function readIds(key: string): string[] {
  try {
    const parsed: unknown = JSON.parse(localStorage.getItem(key) ?? '[]');
    return Array.isArray(parsed) ? parsed.filter((v) => typeof v === 'string') : [];
  } catch {
    return [];
  }
}

function saveIds(key: string, ids: string[]): void {
  try {
    localStorage.setItem(key, JSON.stringify(ids));
  } catch {
    // Private windows may refuse storage; the marks then last until the page reloads.
  }
}

/** Load the marks of whoever is signed in now. */
export function useRailViewer(viewer: string | null): void {
  activityRail.viewer = viewer;
  activityRail.dismissed = readIds(dismissedKey(viewer));
  activityRail.seen = readIds(seenKey(viewer));
}

/** New cards from the shell; cleared ids of cards that are gone are forgotten (web's prune). */
export function setRail(items: RailItem[], history: RailItem[]): void {
  activityRail.items = items;
  activityRail.history = history;
  activityRail.loaded = true;
  const kept = activityRail.dismissed.filter((id) => items.some((i) => i.id === id));
  if (kept.length !== activityRail.dismissed.length) setDismissed(kept);
}

export function clearRail(): void {
  activityRail.items = [];
  activityRail.history = [];
  activityRail.loaded = false;
}

function setDismissed(ids: string[]): void {
  activityRail.dismissed = ids;
  saveIds(dismissedKey(activityRail.viewer), ids);
}

export function dismissRailItem(id: string): void {
  if (!activityRail.dismissed.includes(id)) setDismissed([...activityRail.dismissed, id]);
}

export function restoreRailItem(id: string): void {
  setDismissed(activityRail.dismissed.filter((d) => d !== id));
}

export function restoreAllRailItems(): void {
  setDismissed([]);
}

export function markRailItemSeen(id: string): void {
  if (activityRail.seen.includes(id)) return;
  activityRail.seen = [...activityRail.seen, id];
  saveIds(seenKey(activityRail.viewer), activityRail.seen);
}
