import type { BotReview } from '#lib/domain/botReview.ts';
import { ownCopy, theirCopy, type BotNotice } from '#lib/domain/botReviewNotifications.ts';
import { getBotReview } from '#lib/services/botReview.ts';
import { namesOf } from '#lib/services/people.ts';
import { botReviewNotice } from '#lib/services/pushNotices.ts';

/**
 * Push for a bot review this device's mark or vote just opened or closed (ADR 0108): the
 * account hears its own copy, and whoever had marked it hears theirs, as the cards say.
 */
export async function tellReview(account: string, before: BotReview | undefined): Promise<void> {
  if (!before) return;
  const after = await getBotReview(account);
  const kind: BotNotice['kind'] | null =
    before.status === 'none' && after.status === 'open'
      ? 'opened'
      : before.status === 'open' && after.status === 'upheld'
        ? 'upheld'
        : before.status === 'open' && after.status === 'none'
          ? 'cleared'
          : null;
  if (!kind) return;
  const n: BotNotice = {
    account,
    kind,
    isAppeal: after.isAppeal,
    at: Date.now() * 1000,
    reReviewFrom: after.reReviewFrom === null ? null : after.reReviewFrom * 1000,
  };
  const name = (await namesOf([account]))[account] ?? 'Unnamed';
  const markers = [...new Set([...before.markers, ...after.markers])].filter((m) => m !== account);
  await botReviewNotice([account], 'bot-review', ownCopy(n), account);
  await botReviewNotice(markers, 'bot-review-result', theirCopy(n, name), account);
}
