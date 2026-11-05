import { shortDate } from './botReview.ts';
import type { NotificationItem } from './notifications.ts';

type Unread = Omit<NotificationItem, 'isUnread'>;

/** One opening or closing of a review the viewer is part of (identity `my_bot_review_notices`).
 * Times are Holochain micros. */
export interface BotNotice {
  account: string;
  kind: 'opened' | 'upheld' | 'cleared';
  isAppeal: boolean;
  at: number;
  reReviewFrom: number | null;
}

/** web-backend `bot_reviews.py`'s `bot-review` (to the account) and `bot-review-result` (to
 * whoever marked or voted) cards: "Account", no actor, opening the account's profile. */
export function botReviewNotifications(
  notices: BotNotice[],
  me: string,
  names: Record<string, string>,
): Unread[] {
  return notices.map((n) => {
    const own = n.account === me;
    const name = names[n.account] ?? 'Unnamed';
    const [title, body] = own ? ownCopy(n) : theirCopy(n, name);
    return {
      id: `bot-review:${n.account}:${n.kind}:${n.at}`,
      kind: own ? 'bot-review' : 'bot-review-result',
      subject: 'account',
      seed: '',
      // web's account cards take the project accent (`mapSubjectKind` falls back to it).
      projectMode: 'productive',
      actor: '',
      title,
      body,
      lead: null,
      target: { profile: n.account },
      createdAt: n.at,
    };
  });
}

function ownCopy(n: BotNotice): [string, string] {
  if (n.kind === 'opened') {
    return [
      'Members asked for a bot review of your account',
      'Full members are voting on whether this account belongs to a real person. You can keep posting while the review is open.',
    ];
  }
  if (n.kind === 'upheld') {
    const from = shortDate(Math.floor((n.reReviewFrom ?? n.at) / 1000));
    return [
      'A review found this account is a bot',
      `The account is now read-only. You can ask for one re-review from ${from}.`,
    ];
  }
  return [
    'The bot review cleared your account',
    'Members agreed this account belongs to a real person.',
  ];
}

function theirCopy(n: BotNotice, name: string): [string, string] {
  if (n.kind === 'opened') {
    return [
      `A bot review of ${name} is open`,
      `Three full members marked ${name}, so full members now vote Real person or Bot on their profile for up to 7 days.`,
    ];
  }
  if (n.kind === 'upheld') {
    return [
      `The review found ${name} is a bot`,
      `${name} is now read-only, and their votes no longer count.`,
    ];
  }
  return [
    `The review cleared ${name}`,
    `Members agreed ${name} is a real person. Bot marks on them were cleared.`,
  ];
}
