/** Platform feedback (web's Feedback page, ADR 0054). Times are Holochain micros. */

export type FeedbackKind = 'bug' | 'suggestion';
export type FeedbackFilter = 'all' | 'bugs' | 'suggestions';
export type FeedbackSort = 'trending' | 'recent';
export type FeedbackVote = 'up' | 'down' | null;

export interface FeedbackItem {
  id: string;
  kind: FeedbackKind;
  title: string;
  description: string;
  author: string;
  createdAt: number;
  upvotes: number;
  downvotes: number;
  myVote: FeedbackVote;
  lastActivity: number;
}

export interface FeedbackInput {
  kind: FeedbackKind;
  title: string;
  description: string;
}

// Mirrors the directory's validation (identity_integrity::feedback).
export const FEEDBACK_TITLE_MAX = 200;
export const FEEDBACK_DESCRIPTION_MAX = 5000;

export function approvalPercent(item: FeedbackItem): number {
  const total = item.upvotes + item.downvotes;
  return total > 0 ? (item.upvotes / total) * 100 : 0;
}

/** web-backend `_feedback_trending_score`: votes, decayed by hours since the last activity. */
export function trendingScore(item: FeedbackItem, nowMicros: number): number {
  const hours = Math.max((nowMicros - item.lastActivity) / 3_600_000_000, 0);
  const base = 4 + Math.min(item.upvotes, 50) * 2 - Math.min(item.downvotes, 20);
  return base / (1 + hours / 36);
}

export function shownFeedback(
  items: FeedbackItem[],
  filter: FeedbackFilter,
  sort: FeedbackSort,
  nowMicros: number,
): FeedbackItem[] {
  const kept = items.filter(
    (i) =>
      filter === 'all' ||
      (filter === 'bugs' && i.kind === 'bug') ||
      (filter === 'suggestions' && i.kind === 'suggestion'),
  );
  return sort === 'recent'
    ? kept.sort((a, b) => b.createdAt - a.createdAt)
    : kept.sort((a, b) => trendingScore(b, nowMicros) - trendingScore(a, nowMicros));
}

/** The item as it reads once the caller's vote changes (shown before the write lands). */
export function withVote(item: FeedbackItem, vote: FeedbackVote): FeedbackItem {
  const up = item.upvotes - (item.myVote === 'up' ? 1 : 0) + (vote === 'up' ? 1 : 0);
  const down = item.downvotes - (item.myVote === 'down' ? 1 : 0) + (vote === 'down' ? 1 : 0);
  return { ...item, upvotes: up, downvotes: down, myVote: vote };
}

export function emptyFeedbackCopy(filter: FeedbackFilter): string {
  if (filter === 'bugs') return 'No bugs have been submitted yet.';
  if (filter === 'suggestions') return 'No suggestions have been submitted yet.';
  return 'No feedback has been submitted yet.';
}
