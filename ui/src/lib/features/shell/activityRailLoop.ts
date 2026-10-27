import { startVisibilityPoll } from '#lib/components/actions/visibilityPoll.ts';
import { myActivityRail } from '#lib/services/activityRail.ts';
import { clearRail, setRail, useRailViewer } from '#lib/stores/activityRail.svelte.ts';

let refreshNow: (() => Promise<void>) | null = null;

/**
 * Keep the right rail's cards fresh while `agent` is signed in (web reloads its bootstrap
 * rail on focus and after changes). A failed round keeps the old cards.
 */
export function startActivityRail(agent: string): () => void {
  useRailViewer(agent);
  const refresh = async (): Promise<void> => {
    try {
      const rail = await myActivityRail();
      setRail(rail.items, rail.history);
    } catch {
      // The next round tries again.
    }
  };
  refreshNow = refresh;
  void refresh();
  const stop = startVisibilityPoll(refresh, { activeMs: 60_000 });
  return () => {
    stop();
    refreshNow = null;
    clearRail();
  };
}

/** web's requestActivityRailRefresh: opening the rail reloads it. */
export function refreshActivityRail(): void {
  void refreshNow?.();
}
