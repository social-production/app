import { t } from '#lib/i18n/index.ts';
import { shortDate } from './botReview.ts';
import type { NotificationItem } from './notifications.ts';

type Unread = Omit<NotificationItem, 'isUnread'>;

/** One opening or closing of a review the viewer is part of (identity `my_bot_review_notices`).
 * Times are Holochain micros. */
export interface BotNotice {
  account: string;
  /** `vote`: a review of someone you share a space with opened (identity
   * `bot_review_vote_notices`). */
  kind: 'opened' | 'upheld' | 'cleared' | 'vote';
  isAppeal: boolean;
  at: number;
  reReviewFrom: number | null;
}

/** web-backend `bot_reviews.py`'s `bot-review` (to the account), `bot-review-result` (to
 * whoever marked or voted) and `bot-review-vote` (to people sharing a space) cards: "Account", no actor, opening the account's profile. */
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
      kind: n.kind === 'vote' ? 'bot-review-vote' : own ? 'bot-review' : 'bot-review-result',
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

export function ownCopy(n: BotNotice): [string, string] {
  if (n.kind === 'opened') {
    return [
      t('Members asked for a bot review of your account'),
      t(
        'People are voting on whether this account belongs to a real person. You can keep posting while the review is open.',
      ),
    ];
  }
  if (n.kind === 'upheld') {
    const from = shortDate(Math.floor((n.reReviewFrom ?? n.at) / 1000));
    return [
      'A review found this account is a bot',
      t('The account is now read-only. You can ask for one re-review from {from}.', { from }),
    ];
  }
  return [
    'The bot review cleared your account',
    'Members agreed this account belongs to a real person.',
  ];
}

export function theirCopy(n: BotNotice, name: string): [string, string] {
  if (n.kind === 'vote') {
    return [
      t('Is {name} a real person?', { name }),
      t(
        '{name} is in a bot review, and you share a space with them. If you know them, vote Real person or Bot on their profile.',
        { name },
      ),
    ];
  }
  if (n.kind === 'opened') {
    return [
      t('A bot review of {name} is open', { name }),
      t(
        'Three full members marked {name}, so anyone can now vote Real person or Bot on their profile for up to 7 days.',
        { name },
      ),
    ];
  }
  if (n.kind === 'upheld') {
    return [
      t('The review found {name} is a bot', { name }),
      t('{name} is now read-only, and their votes no longer count.', { name }),
    ];
  }
  return [
    t('The review cleared {name}', { name }),
    t('Members agreed {name} is a real person. Bot marks on them were cleared.', { name }),
  ];
}
