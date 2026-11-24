import { getContext, setContext } from 'svelte';

import type { VoteDirection } from '#lib/domain/discussions.ts';

/** Votes on a thread's or post's comment from a feed row (owner, 2026-11-24). The shell
 * provides it (CODE_RULES: components don't read services). */
export interface CommentLike {
  vote: (space: string, message: string, direction: VoteDirection) => Promise<void>;
}

const KEY = Symbol('comment-like');

export function provideCommentLike(source: CommentLike): void {
  setContext(KEY, source);
}

export function commentLike(): CommentLike | undefined {
  return getContext<CommentLike | undefined>(KEY);
}
