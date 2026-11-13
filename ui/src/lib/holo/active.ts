/** The weekly active count (ADR 0095): the first meaningful action of each week also writes
 * this person's `Active` link in the directory (the zome skips it if one exists this week). */

/** Zome functions that count as taking part (web's `record_meaningful_action` callers). */
const MEANINGFUL = new Set([
  'vote',
  'propose',
  'challenge',
  'create_charter',
  'post_message',
  'set_signal',
  'take_role',
  'join',
  'rate',
  'rate_activity',
  'rate_value',
  'add_value',
  'create_activity',
  'book_request',
  'vote_message',
  'vote_report',
  'vouch',
  'create_feedback',
  'vote_feedback',
  'vote_bot_review',
]);

const WEEK_MS = 7 * 24 * 60 * 60 * 1000;
/** Weeks start on Monday 00:00 UTC; 1970-01-01 was a Thursday (`sp_active::week_of`). */
const weekOf = (ms: number): number => Math.floor((ms - 4 * 24 * 60 * 60 * 1000) / WEEK_MS);

let recordedWeek = -1;

/** After a successful call: write this week's link once, in the background. */
export function noteCall(fn: string, record: () => Promise<unknown>): void {
  if (!MEANINGFUL.has(fn)) return;
  const week = weekOf(Date.now());
  if (week === recordedWeek) return;
  recordedWeek = week;
  void record().catch(() => {
    recordedWeek = -1; // try again on the next action
  });
}
