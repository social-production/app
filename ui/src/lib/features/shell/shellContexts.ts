import { providePlaceOnMap } from '#lib/components/map/placeOnMap.ts';
import { openPlaceOnMap } from '#lib/stores/mapFocus.svelte.ts';

import { providePaused } from './pausedProvider.svelte.ts';
import { provideReports } from './reportActionsProvider.ts';

/** Contexts the shell gives every card and author line below it. */
export function provideShellContexts(): void {
  provideReports();
  providePaused();
  providePlaceOnMap({ show: (label) => openPlaceOnMap(label) });
}
