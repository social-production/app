/** The weekly active count (ADR 0095, rolling since ADR 0121): a meaningful action also writes
 * this person's `Active` link in the directory, at most one an hour (the zome skips it if one
 * exists within the hour). */

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

const HOUR_MS = 60 * 60 * 1000;

let recordedAt = -Infinity;

/** After a successful call: write a link at most once an hour, in the background. */
export function noteCall(fn: string, record: () => Promise<unknown>): void {
  if (!MEANINGFUL.has(fn)) return;
  const now = Date.now();
  if (now - recordedAt < HOUR_MS) return;
  recordedAt = now;
  // A little later: the action that got here is often followed at once by another write to
  // the directory (publishing lists the project), and two writes at once refuse one of them.
  setTimeout(() => {
    void record().catch(() => {
      recordedAt = -Infinity; // try again on the next action
    });
  }, 5000);
}
