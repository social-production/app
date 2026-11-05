import { onDestroy } from 'svelte';

import { providePausedAgents } from '#lib/components/people/pausedAgents.ts';
import { pausedAgents } from '#lib/services/botReview.ts';

/** The gateway refreshes its people index every minute; the badge follows every 5. */
const REFRESH_MS = 5 * 60_000;

/** Give every author line below the shell web's "Paused" badge list (ADR 0074). */
export function providePaused(): void {
  let paused = $state<readonly string[]>([]);
  const refresh = () => void pausedAgents().then((list) => (paused = list));
  refresh();
  const timer = setInterval(refresh, REFRESH_MS);
  onDestroy(() => clearInterval(timer));
  providePausedAgents({ has: (agent) => paused.includes(agent) });
}
