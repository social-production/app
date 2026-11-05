import { getContext, setContext } from 'svelte';

/**
 * Who a bot review paused (ADR 0074), for web's "Paused" badge after author names. Components
 * stay presentational (CODE_RULES): the shell provides the list (`providePaused`).
 */
export interface PausedAgents {
  has: (agent: string) => boolean;
}

const KEY = Symbol('paused-agents');
const NONE: PausedAgents = { has: () => false };

export function providePausedAgents(paused: PausedAgents): void {
  setContext(KEY, paused);
}

export function pausedAgents(): PausedAgents {
  return getContext<PausedAgents | undefined>(KEY) ?? NONE;
}
