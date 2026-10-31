import type { Contact } from '#lib/domain/messages.ts';
import type { Charter } from '#lib/domain/projects.ts';
import { shareMessage } from '#lib/domain/share.ts';
import {
  POST_MODE,
  THREAD_MODE,
  commentTree,
  postTitle,
  type CommentVotes,
  type DiscussionComment,
  type VoteDirection,
} from '#lib/domain/discussions.ts';
import { callCell, hashToString, stringToHash, type ActionHash } from '#lib/holo/index.ts';

import { listMessages, postMessage, queued } from './chat.ts';
import { sendNewChat } from './messages.ts';
import { cellFor, createProject } from './projects.ts';

/** Threads and posts (ADR 0070): commons spaces whose chat holds their comments. */
function charter(mode: string, title: string, body: string, tags: string[]): Charter {
  return {
    title,
    description: body.trim(),
    mode: mode as Charter['mode'],
    subtype: 'standard',
    location_label: '',
    phase: '',
    tags,
  };
}

/** web's createThread: a title, a body and at least one channel or community tag. */
export function createThread(title: string, body: string, tags: string[]): Promise<string> {
  return createProject(charter(THREAD_MODE, title.trim(), body, tags));
}

/** web's createPost: a public note on your personal feed. */
export function createPost(body: string): Promise<string> {
  return createProject(charter(POST_MODE, postTitle(body), body, []));
}

interface VoteWire {
  id: ActionHash;
  count: number;
  mine: VoteDirection;
}

/** The comments (newest 200) as web's tree, each with its score and your vote. */
export async function listComments(id: string): Promise<DiscussionComment[]> {
  const messages = await listMessages(id);
  const votes: Record<string, CommentVotes> = {};
  if (messages.length) {
    const wire = await callCell<VoteWire[]>(
      await cellFor(id),
      'discussion',
      'message_votes',
      messages.map((m) => stringToHash(m.id)),
    );
    for (const v of wire) votes[hashToString(v.id)] = { count: v.count, mine: v.mine };
  }
  return commentTree(messages, votes);
}

/** web's addComment: a comment, or a reply to one. */
export function addComment(id: string, body: string, parent: string | null): Promise<string> {
  return postMessage(id, body.trim(), parent);
}

/** web's setVote on a comment: 1 up, -1 down, 0 takes it back. */
export async function voteComment(
  id: string,
  comment: string,
  direction: VoteDirection,
): Promise<void> {
  const cell = await cellFor(id);
  await queued(() =>
    callCell<null>(cell, 'discussion', 'vote_message', {
      message: stringToHash(comment),
      direction,
    }),
  );
}

/** shares.py for a thread or post: the link goes into your direct chat with them. */
export async function shareDiscussion(
  to: Contact,
  kind: 'thread' | 'post',
  title: string,
  url: string,
): Promise<void> {
  await sendNewChat({
    kind: 'direct',
    members: [to],
    title: '',
    body: shareMessage(kind, title, url),
  });
}
