import { inChat, type CommentActivity, type CommentCounts } from '#lib/domain/commentNotes.ts';
import type { DiscussionComment } from '#lib/domain/discussions.ts';

import { listComments } from './discussions.ts';

const repliesUnder = (c: DiscussionComment): number =>
  c.replies.reduce((n, r) => n + 1 + repliesUnder(r), 0);

function index(tree: DiscussionComment[], into: Map<string, CommentCounts>): void {
  for (const c of tree) {
    into.set(c.id, {
      likes: Math.max(0, c.voteCount),
      replies: repliesUnder(c),
      liked: c.activeVote === 1,
    });
    index(c.replies, into);
  }
}

/**
 * Comments on threads and posts with their likes and replies, as the thread shows them
 * (owner, 2026-11-24: a profile's comments had neither). One read per thread; a thread that
 * can't be read leaves its comments without counts. Chat comments have no likes.
 */
export async function withCounts(comments: CommentActivity[]): Promise<CommentActivity[]> {
  const spaces = [...new Set(comments.filter((c) => !inChat(c.space.mode)).map((c) => c.space.id))];
  const counts = new Map<string, CommentCounts>();
  await Promise.all(
    spaces.map((id) =>
      listComments(id).then(
        (tree) => index(tree, counts),
        () => undefined,
      ),
    ),
  );
  return comments.map((c) => {
    const found = counts.get(c.message);
    return found ? { ...c, counts: found } : c;
  });
}
