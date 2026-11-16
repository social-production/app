import { categoryOf } from '#lib/domain/notificationCategories.ts';
import type { NotificationKind } from '#lib/domain/notifications.ts';
import { sendNotice } from '#lib/holo/index.ts';

export { enableNotices, onNotice } from '#lib/holo/index.ts';

import { viewer } from './people.ts';

/** The longest body a notice carries (the zome refuses more than 500 characters). */
const BODY_MAX = 280;

function clip(text: string): string {
  const flat = text.replace(/\s+/g, ' ').trim();
  return flat.length > BODY_MAX ? `${flat.slice(0, BODY_MAX - 1)}…` : flat;
}

/**
 * Tell `to` about a card this write just made for them (ADR 0108), worded as the card is.
 * Never throws: the card is on the chain either way, and the notice is only a nudge.
 */
export async function notify(
  to: (string | null | undefined)[],
  kind: NotificationKind,
  title: (me: string) => string,
  body: string,
  link: string,
): Promise<void> {
  try {
    const me = await viewer();
    const people = [...new Set(to.filter((a): a is string => !!a && a !== me.agent))];
    if (!people.length) return;
    await sendNotice(people, {
      kind,
      category: categoryOf[kind],
      title: title(me.name || 'Someone'),
      body: clip(body),
      link,
    });
  } catch {
    /* Best effort. */
  }
}
