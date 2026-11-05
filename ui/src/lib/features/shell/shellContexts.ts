import { providePaused } from './pausedProvider.svelte.ts';
import { provideReports } from './reportActionsProvider.ts';

/** Contexts the shell gives every card and author line below it. */
export function provideShellContexts(): void {
  provideReports();
  providePaused();
}
