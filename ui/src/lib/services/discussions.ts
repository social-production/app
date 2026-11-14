import type { ChatAttachment } from '#lib/domain/chat.ts';
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

import { projectAttachmentBlob, uploadProjectFiles } from './attachments.ts';
import { listMessages, postMessage, queued, type AttachmentWire } from './chat.ts';
import { getMessageModeration } from './moderation.ts';
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

/** `media::PHOTOS_MAX`: photos on one post (ADR 0101). */
export const POST_PHOTOS_MAX = 4;

/**
 * web's createPost: a public note on your personal feed, with up to four photos (ADR 0101).
 * The photos are written after the post; if that fails the post stands without them.
 */
export async function createPost(body: string, photos: File[] = []): Promise<string> {
  const id = await createProject(charter(POST_MODE, postTitle(body), body, []));
  if (photos.length) {
    await queued(async () => {
      const attachments = await uploadProjectFiles(id, photos);
      await callCell<ActionHash>(await cellFor(id), 'discussion', 'set_post_media', attachments);
    });
  }
  return id;
}

const photoLists = new Map<string, Promise<ChatAttachment[]>>();

/** A post's photos, read once per post (they never change); an unreachable read is retried. */
export function postPhotos(id: string): Promise<ChatAttachment[]> {
  let list = photoLists.get(id);
  if (!list) {
    list = cellFor(id)
      .then((cell) => callCell<AttachmentWire[]>(cell, 'discussion', 'post_media', null))
      .then((wire) => wire.map((a) => ({ ...a, chunks: a.chunks.map(hashToString) })));
    photoLists.set(id, list);
    list.catch(() => photoLists.delete(id));
  }
  return list;
}

const photoUrls = new Map<string, Promise<string>>();

/** One photo of a post as an object URL, kept for the session. */
export function postPhotoUrl(id: string, photo: ChatAttachment): Promise<string> {
  const key = photo.chunks.join('.');
  let url = photoUrls.get(key);
  if (!url) {
    url = projectAttachmentBlob(id, photo).then((blob) => URL.createObjectURL(blob));
    photoUrls.set(key, url);
    url.catch(() => photoUrls.delete(key));
  }
  return url;
}

interface VoteWire {
  id: ActionHash;
  count: number;
  mine: VoteDirection;
}

/** The comments (newest 200) as web's tree, each with its score, your vote and its report. */
export async function listComments(id: string): Promise<DiscussionComment[]> {
  const messages = await listMessages(id);
  const votes: Record<string, CommentVotes> = {};
  if (!messages.length) return [];
  const ids = messages.map((m) => m.id);
  const [wire, moderation] = await Promise.all([
    callCell<VoteWire[]>(await cellFor(id), 'discussion', 'message_votes', ids.map(stringToHash)),
    // An unreadable report leaves the comments shown as they are.
    getMessageModeration(id, ids).catch(() => ({})),
  ]);
  for (const v of wire) votes[hashToString(v.id)] = { count: v.count, mine: v.mine };
  return commentTree(messages, votes, moderation);
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
