import { toAvatar, toCover } from '#lib/components/people/avatar.ts';
import { putScopeImage, scopeImageUrl } from '#lib/services/scopeImages.ts';

/**
 * A scope's picture and cover as image URLs (ADR 0109), and the new ones Settings stages
 * before they go to a vote (or straight in, in an organizer-run community).
 */
export class ScopeArt {
  pictureUrl = $state<string | null>(null);
  coverUrl = $state<string | null>(null);
  /** Staged in Settings: the stored image's hash and a preview. */
  newPicture = $state<{ hash: string; url: string } | null>(null);
  newCover = $state<{ hash: string; url: string } | null>(null);
  busy = $state(false);

  /** Show the charter's current picture and cover. */
  async show(picture: string | null | undefined, cover: string | null | undefined): Promise<void> {
    const [p, c] = await Promise.all([
      picture ? scopeImageUrl(picture) : null,
      cover ? scopeImageUrl(cover) : null,
    ]);
    this.pictureUrl = p;
    this.coverUrl = c;
  }

  /** Resize and store a new picture (square) or cover (3:1), staged until saved. */
  async stage(slot: 'picture' | 'cover', file: File): Promise<void> {
    this.busy = true;
    try {
      const image = slot === 'picture' ? await toAvatar(file) : await toCover(file);
      const hash = await putScopeImage(image);
      const url = URL.createObjectURL(
        new Blob([new Uint8Array(image.bytes)], { type: image.mime }),
      );
      if (slot === 'picture') this.newPicture = { hash, url };
      else this.newCover = { hash, url };
    } finally {
      this.busy = false;
    }
  }

  clearStaged(): void {
    this.newPicture = null;
    this.newCover = null;
  }
}

/** One image of a decision (a proposed picture or cover) as a URL. */
export function imageUrl(hash: string | null): Promise<string | null> {
  return hash ? scopeImageUrl(hash) : Promise.resolve(null);
}
