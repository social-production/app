import type {
  FeedbackComment,
  FeedbackInput,
  FeedbackItem,
  FeedbackVote,
} from '#lib/domain/feedback.ts';
import {
  callZome,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

const identity = <T>(fn: string, payload?: unknown) =>
  callZome<T>('directory', 'identity', fn, payload);

interface ItemWire {
  id: ActionHash;
  kind: FeedbackItem['kind'];
  title: string;
  description: string;
  author: AgentPubKey;
  created_at: number;
  upvotes: number;
  downvotes: number;
  my_vote: FeedbackVote;
  last_activity: number;
  comments: number;
}

const toItem = (w: ItemWire): FeedbackItem => ({
  id: hashToString(w.id),
  kind: w.kind,
  title: w.title,
  description: w.description,
  author: hashToString(w.author),
  createdAt: w.created_at,
  upvotes: w.upvotes,
  downvotes: w.downvotes,
  myVote: w.my_vote,
  lastActivity: w.last_activity,
  comments: w.comments,
});

export async function listFeedback(): Promise<FeedbackItem[]> {
  return (await identity<ItemWire[]>('list_feedback')).map(toItem);
}

export async function getFeedback(id: string): Promise<FeedbackItem | null> {
  const w = await identity<ItemWire | null>('get_feedback', stringToHash(id));
  return w ? toItem(w) : null;
}

export async function createFeedback(input: FeedbackInput): Promise<FeedbackItem> {
  return toItem(await identity<ItemWire>('create_feedback', input));
}

/** Set or clear (`null`) the caller's vote. */
export async function voteFeedback(
  id: string,
  direction: FeedbackVote,
): Promise<FeedbackItem | null> {
  const w = await identity<ItemWire | null>('vote_feedback', {
    id: stringToHash(id),
    direction,
  });
  return w ? toItem(w) : null;
}

interface CommentWire {
  id: ActionHash;
  author: AgentPubKey;
  body: string;
  reply_to: ActionHash | null;
  created_at: number;
  likes: number;
  liked: boolean;
}

const toComment = (w: CommentWire): FeedbackComment => ({
  id: hashToString(w.id),
  author: hashToString(w.author),
  body: w.body,
  replyTo: w.reply_to ? hashToString(w.reply_to) : null,
  createdAt: w.created_at,
  likes: w.likes,
  liked: w.liked,
});

/** An item's comments, oldest first. */
export async function feedbackComments(id: string): Promise<FeedbackComment[]> {
  return (await identity<CommentWire[]>('list_feedback_comments', stringToHash(id))).map(toComment);
}

export async function addFeedbackComment(
  id: string,
  body: string,
  replyTo: string | null,
): Promise<FeedbackComment> {
  return toComment(
    await identity<CommentWire>('add_feedback_comment', {
      feedback: stringToHash(id),
      body,
      reply_to: replyTo ? stringToHash(replyTo) : null,
    }),
  );
}

export async function likeFeedbackComment(
  id: string,
  like: boolean,
): Promise<FeedbackComment | null> {
  const w = await identity<CommentWire | null>('like_feedback_comment', {
    id: stringToHash(id),
    like,
  });
  return w ? toComment(w) : null;
}
