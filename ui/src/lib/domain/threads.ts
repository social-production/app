/** Mirrors the project DNA's discussion threads (ADR 0041). Times are Holochain micros. */

/** sp_core::discussion limits. */
export const TITLE_MAX = 200;
export const BODY_MAX = 20_000;
export const COMMENT_MAX = 4_000;

/** web's VoteDirection: up, none, down. */
export type VoteDirection = -1 | 0 | 1;

export type ThreadSort = 'trending' | 'recent';
export type ThreadWindow = 'today' | 'week' | 'month' | 'all';
/** web's thread-page comment sort. */
export type CommentSort = 'oldest' | 'newest' | 'top';

export interface ThreadItem {
  id: string;
  author: string;
  title: string;
  body: string;
  createdAt: number;
  lastActivityAt: number;
  voteCount: number;
  myVote: VoteDirection;
  commentCount: number;
}

export interface ThreadComment {
  id: string;
  author: string;
  parent: string | null;
  body: string;
  createdAt: number;
  voteCount: number;
  myVote: VoteDirection;
}

/** A comment with its replies, as web's `get_comments` nests them. */
export interface CommentNode extends ThreadComment {
  authorName: string;
  replies: CommentNode[];
}

export interface ThreadDetail {
  thread: ThreadItem;
  comments: ThreadComment[];
}

/** web's `applyVoteTarget`: the new vote and the count it gives. */
export function applyVote(
  active: VoteDirection,
  count: number,
  target: VoteDirection,
): { myVote: VoteDirection; voteCount: number } {
  return { myVote: target, voteCount: count + target - active };
}

/**
 * Nest comments under their parents (web's `build_tree`), oldest first at every level. A reply
 * whose parent isn't in the page shows at the top level rather than disappearing.
 */
export function commentTree(
  comments: ThreadComment[],
  nameOf: (agent: string) => string,
): CommentNode[] {
  const nodes = new Map<string, CommentNode>();
  for (const c of [...comments].sort((a, b) => a.createdAt - b.createdAt)) {
    nodes.set(c.id, { ...c, authorName: nameOf(c.author), replies: [] });
  }
  const roots: CommentNode[] = [];
  for (const node of nodes.values()) {
    const parent = node.parent ? nodes.get(node.parent) : undefined;
    if (parent) parent.replies.push(node);
    else roots.push(node);
  }
  return roots;
}

/** web's DiscussionPanel sort: it orders the top-level comments only. */
export function sortComments(roots: CommentNode[], mode: CommentSort): CommentNode[] {
  return [...roots].sort((a, b) => {
    if (mode === 'top') return b.voteCount - a.voteCount || b.createdAt - a.createdAt;
    if (mode === 'newest') return b.createdAt - a.createdAt;
    return a.createdAt - b.createdAt;
  });
}

/** web's comment-count label. */
export function commentLabel(count: number): string {
  return `${count} ${count === 1 ? 'comment' : 'comments'}`;
}
