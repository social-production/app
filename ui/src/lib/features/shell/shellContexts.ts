import { providePostPhotoSource } from '#lib/components/cards/postPhotoSource.ts';
import { provideCommentLike } from '#lib/components/cards/commentLike.ts';
import { provideQuoteSource } from '#lib/components/cards/quoteSource.ts';
import { providePlaceOnMap } from '#lib/components/map/placeOnMap.ts';
import { provideShareLink } from '#lib/components/overlay/shareLink.ts';
import { postPhotoUrl, postPhotos, voteComment } from '#lib/services/discussions.ts';
import { quotedSpace } from '#lib/services/profileNotes.ts';
import {
  SCOPE_PICTURES,
  SPACE_PICTURES,
  type ScopePictureOf,
  type SpacePictureOf,
} from '#lib/domain/scopes.ts';
import { openPlaceOnMap } from '#lib/stores/mapFocus.svelte.ts';

import { provideAvatars } from './avatarProvider.svelte.ts';
import { providePaused } from './pausedProvider.svelte.ts';
import { provideReports } from './reportActionsProvider.ts';
import { ScopePictures } from './scopePictures.svelte.ts';
import { shareHost } from './shareHost.svelte.ts';
import { setContext } from 'svelte';

/** Contexts the shell gives every card and author line below it. */
export function provideShellContexts(): void {
  // Tags show their channel's or community's picture (ADR 0109); spaces their own (ADR 0117).
  const pictures = new ScopePictures();
  setContext<ScopePictureOf>(SCOPE_PICTURES, pictures.of);
  setContext<SpacePictureOf>(SPACE_PICTURES, pictures.ofHash);
  provideReports();
  providePaused();
  provideAvatars();
  providePlaceOnMap({ show: (label, select) => openPlaceOnMap(label, null, select ?? null) });
  providePostPhotoSource({ list: postPhotos, url: postPhotoUrl });
  provideQuoteSource({ summary: quotedSpace });
  provideCommentLike({ vote: voteComment });
  provideShareLink({ open: (subject) => (shareHost.subject = subject) });
}
