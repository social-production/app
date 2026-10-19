import type {
  ThreadComment,
  ThreadDetail,
  ThreadItem,
  ThreadSort,
  ThreadWindow,
  VoteDirection,
} from '#lib/domain/threads.ts';
import {
  callCell,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { cellOf } from './projects.ts';

interface ThreadWire {
  id: ActionHash;
  author: AgentPubKey;
  title: string;
  body: string;
  created_at: number;
  last_activity_at: number;
  vote_count: number;
  my_vote: number;
  comment_count: number;
}

interface CommentWire {
  id: ActionHash;
  author: AgentPubKey;
  parent: ActionHash | null;
  body: string;
  created_at: number;
  vote_count: number;
  my_vote: number;
}

const discussion = async <T>(projectId: string, fn: string, payload?: unknown): Promise<T> => {
  const { cell } = await cellOf(projectId);
  return callCell<T>(cell, 'discussion', fn, payload);
};

const direction = (n: number): VoteDirection => (n > 0 ? 1 : n < 0 ? -1 : 0);

function thread(w: ThreadWire): ThreadItem {
  return {
    id: hashToString(w.id),
    author: hashToString(w.author),
    title: w.title,
    body: w.body,
    createdAt: w.created_at,
    lastActivityAt: w.last_activity_at,
    voteCount: w.vote_count,
    myVote: direction(w.my_vote),
    commentCount: w.comment_count,
  };
}

function comment(w: CommentWire): ThreadComment {
  return {
    id: hashToString(w.id),
    author: hashToString(w.author),
    parent: w.parent ? hashToString(w.parent) : null,
    body: w.body,
    createdAt: w.created_at,
    voteCount: w.vote_count,
    myVote: direction(w.my_vote),
  };
}

/** The project's threads, ordered like web's channel feeds (the zome returns at most 200). */
export async function listThreads(
  projectId: string,
  sort: ThreadSort,
  window: ThreadWindow,
): Promise<ThreadItem[]> {
  const wire = await discussion<ThreadWire[]>(projectId, 'list_threads', {
    sort,
    window,
    utc_offset_minutes: -new Date().getTimezoneOffset(),
  });
  return wire.map(thread);
}

export async function getThread(projectId: string, id: string): Promise<ThreadDetail> {
  const wire = await discussion<{ thread: ThreadWire; comments: CommentWire[] }>(
    projectId,
    'get_thread',
    stringToHash(id),
  );
  return { thread: thread(wire.thread), comments: wire.comments.map(comment) };
}

/** Anyone signed in may start a thread, as on web. Returns the new thread's id. */
export async function createThread(
  projectId: string,
  title: string,
  body: string,
): Promise<string> {
  const id = await discussion<ActionHash>(projectId, 'create_thread', { title, body });
  return hashToString(id);
}

export async function addComment(
  projectId: string,
  threadId: string,
  parent: string | null,
  body: string,
): Promise<void> {
  await discussion<ActionHash>(projectId, 'add_comment', {
    thread: stringToHash(threadId),
    parent: parent ? stringToHash(parent) : null,
    body,
  });
}

/** Up, down, or 0 to take the vote back, on a thread or a comment. */
export async function castVote(
  projectId: string,
  target: string,
  vote: VoteDirection,
): Promise<void> {
  await discussion<null>(projectId, 'cast_vote', { target: stringToHash(target), direction: vote });
}
