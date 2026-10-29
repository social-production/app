import { myScopes } from '#lib/services/scopes.ts';
import { showMyScopes } from '#lib/stores/myScopes.svelte.ts';

import { startVisibilityPoll } from '#lib/components/actions/visibilityPoll.ts';

/** Keeps the left rail's channels and communities fresh while signed in (every minute). */
export function startMyScopes(): () => void {
  const refresh = async (): Promise<void> => {
    try {
      showMyScopes(await myScopes());
    } catch {
      // keep the last lists; the next round tries again
    }
  };
  void refresh();
  const stop = startVisibilityPoll(refresh, { activeMs: 60_000 });
  return () => {
    stop();
    showMyScopes([]);
  };
}
