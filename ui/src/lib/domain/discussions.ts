import type { ChatMessage } from './chat.ts';

/** Threads and posts (ADR 0070): commons spaces with these charter modes. */
export const THREAD_MODE = 'thread';
export const POST_MODE = 'post';

/** web's CommentSort. */
export type CommentSort = 'oldest' | 'newest' | 'top';

/** web's VoteDirection: 1 up, -1 down, 0 none. */
export type VoteDirection = -1 | 0 | 1;

/** A comment's score and the viewer's vote (the discussion zome's `VoteView`). */
export interface CommentVotes {
  count: number;
  mine: VoteDirection;
}

/** web's DetailComment: a comment with its votes and nested replies. */
export interface DiscussionComment {
  id: string;
  author: string;
  body: string;
  /** Holochain microseconds. */
  createdAt: number;
  voteCount: number;
  activeVote: VoteDirection;
  replies: DiscussionComment[];
}

/**
 * web-backend's `get_comments` tree: a reply nests under the comment it answers (its
 * `reply_to`); one whose original is gone stays at the top. Replies keep oldest first.
 */
export function commentTree(
  messages: ChatMessage[],
  votes: Record<string, CommentVotes>,
): DiscussionComment[] {
  const ordered = [...messages].sort((a, b) => a.createdAt - b.createdAt);
  const byId = new Map<string, DiscussionComment>();
  for (const m of ordered) {
    const v = votes[m.id];
    byId.set(m.id, {
      id: m.id,
      author: m.author,
      body: m.body,
      createdAt: m.createdAt,
      voteCount: v?.count ?? 0,
      activeVote: v?.mine ?? 0,
      replies: [],
    });
  }
  const roots: DiscussionComment[] = [];
  for (const m of ordered) {
    const node = byId.get(m.id);
    const parent = m.replyTo ? byId.get(m.replyTo.id) : undefined;
    if (!node) continue;
    if (parent && parent !== node) parent.replies.push(node);
    else roots.push(node);
  }
  return roots;
}

/** web's DiscussionPanel sort (top-level comments only). */
export function sortComments(list: DiscussionComment[], mode: CommentSort): DiscussionComment[] {
  return [...list].sort((a, b) => {
    if (mode === 'top') return b.voteCount - a.voteCount || b.createdAt - a.createdAt;
    if (mode === 'newest') return b.createdAt - a.createdAt;
    return a.createdAt - b.createdAt;
  });
}

/** Every comment in the tree, replies included. */
export function countComments(list: DiscussionComment[]): number {
  return list.reduce((n, c) => n + 1 + countComments(c.replies), 0);
}

/** A post's charter title (posts show none): its first line, cut to 80 characters. */
export function postTitle(body: string): string {
  const line = body.trim().split('\n')[0]?.trim() ?? '';
  return [...line].slice(0, 80).join('') || 'Post';
}
