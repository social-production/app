import type { ImageChange } from '#lib/domain/governance.ts';
import type { Avatar } from '#lib/domain/people.ts';
import type { Charter } from '#lib/domain/projects.ts';
import { callZome, hashToString, stringToHash } from '#lib/holo/index.ts';

/**
 * Pictures and covers for channels and communities (ADR 0109): directory `ScopeImage`
 * records, named by hash from the scope's charter and the index's cards. Images never change
 * (a new one is a new hash), so each is fetched once per session.
 */
const identity = <T>(fn: string, payload?: unknown) =>
  callZome<T>('directory', 'identity', fn, payload);

/** Store an image (resized first, as covers are) and return the hash a charter names. */
export async function putScopeImage(image: Avatar): Promise<string> {
  return hashToString(await identity<Uint8Array>('put_scope_image', image));
}

const urls = new Map<string, Promise<string | null>>();

/** A blob URL for the image `hash` names, or null when it can't be found. */
export function scopeImageUrl(hash: string): Promise<string | null> {
  let found = urls.get(hash);
  if (!found) {
    found = identity<Avatar | null>('scope_image', stringToHash(hash)).then(
      (image) =>
        image
          ? URL.createObjectURL(new Blob([new Uint8Array(image.bytes)], { type: image.mime }))
          : null,
      () => null,
    );
    urls.set(hash, found);
  }
  return found;
}

/** The charter as the zome sends it: its picture and cover are raw hashes. */
export type CharterWire = Omit<Charter, 'picture' | 'cover'> & {
  picture?: Uint8Array | null;
  cover?: Uint8Array | null;
};

export function charterIn(c: CharterWire | null): Charter | null {
  if (!c) return null;
  return {
    ...c,
    picture: c.picture ? hashToString(c.picture) : null,
    cover: c.cover ? hashToString(c.cover) : null,
  };
}

export function charterOut(c: Charter): CharterWire {
  return {
    ...c,
    picture: c.picture ? stringToHash(c.picture) : null,
    cover: c.cover ? stringToHash(c.cover) : null,
  };
}

type ArtWire = { Set: Uint8Array } | 'Remove';

/** A proposal's image change as the zome takes it, and back (ADR 0109). */
export function artOut(c: ImageChange | null | undefined): ArtWire | null {
  if (!c) return null;
  return c === 'Remove' ? 'Remove' : { Set: stringToHash(c.Set) };
}

export function artIn(c: ArtWire | ImageChange | null | undefined): ImageChange | null {
  if (!c) return null;
  if (c === 'Remove') return 'Remove';
  return { Set: typeof c.Set === 'string' ? c.Set : hashToString(c.Set) };
}
