import type { FeedbackInput, FeedbackItem, FeedbackVote } from '#lib/domain/feedback.ts';
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
