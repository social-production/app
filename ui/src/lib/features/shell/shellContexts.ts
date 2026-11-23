import { providePostPhotoSource } from '#lib/components/cards/postPhotoSource.ts';
import { provideQuoteSource } from '#lib/components/cards/quoteSource.ts';
import { providePlaceOnMap } from '#lib/components/map/placeOnMap.ts';
import { provideShareLink } from '#lib/components/overlay/shareLink.ts';
import { postPhotoUrl, postPhotos } from '#lib/services/discussions.ts';
import { quotedSpace } from '#lib/services/profileNotes.ts';
import { openPlaceOnMap } from '#lib/stores/mapFocus.svelte.ts';

import { provideAvatars } from './avatarProvider.svelte.ts';
import { providePaused } from './pausedProvider.svelte.ts';
import { provideReports } from './reportActionsProvider.ts';
import { shareHost } from './shareHost.svelte.ts';

/** Contexts the shell gives every card and author line below it. */
export function provideShellContexts(): void {
  provideReports();
  providePaused();
  provideAvatars();
  providePlaceOnMap({ show: (label, select) => openPlaceOnMap(label, null, select ?? null) });
  providePostPhotoSource({ list: postPhotos, url: postPhotoUrl });
  provideQuoteSource({ summary: quotedSpace });
  provideShareLink({ open: (subject) => (shareHost.subject = subject) });
}
