/** A public vote among full members on whether an account is a bot (web's `BotReview`,
 * ADR 0074). Times are milliseconds. */
export interface BotReview {
  status: 'none' | 'open' | 'upheld';
  isAppeal: boolean;
  botVotes: number;
  realVotes: number;
  votesRequired: number;
  closesAt: number | null;
  myVote: 'bot' | 'real' | null;
  /** When a paused account may ask for its one re-review (null once it was used). */
  reReviewFrom: number | null;
  /** Everyone whose bot mark is live. */
  markers: string[];
  /** Until when the viewer can't mark this account again (a cleared review's marker). */
  myCooldownUntil: number | null;
}

/** web's `toLocaleDateString(undefined, { day: 'numeric', month: 'short' })`. */
export function shortDate(ms: number): string {
  return new Date(ms).toLocaleDateString(undefined, { day: 'numeric', month: 'short' });
}
