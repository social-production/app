import { getContext, setContext } from 'svelte';

import type { ChatAttachment } from '#lib/domain/chat.ts';

/**
 * Where a post's photos come from (ADR 0101). Components stay presentational (CODE_RULES):
 * the shell provides the reads (`providePostPhotoSource`).
 */
export interface PostPhotoSource {
  /** The post's photos; empty when it has none. */
  list: (post: string) => Promise<ChatAttachment[]>;
  /** One photo as a URL an `<img>` can show. */
  url: (post: string, photo: ChatAttachment) => Promise<string>;
}

const KEY = Symbol('post-photo-source');
const NONE: PostPhotoSource = {
  list: () => Promise.resolve([]),
  url: () => Promise.reject(new Error('no photo source')),
};

export function providePostPhotoSource(source: PostPhotoSource): void {
  setContext(KEY, source);
}

export function postPhotoSource(): PostPhotoSource {
  return getContext<PostPhotoSource | undefined>(KEY) ?? NONE;
}
