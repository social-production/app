import { startVisibilityPoll } from '#lib/components/actions/visibilityPoll.ts';
import { myNotifications } from '#lib/services/notifications.ts';
import { unreadMessageCount } from '#lib/services/unread.ts';
import { messageBadge } from '#lib/stores/messageBadge.svelte.ts';
import * as bell from '#lib/stores/notifications.svelte.ts';

/**
 * The bell's and the send icon's counts stay fresh while someone is signed in (every 30 s
 * while visible). The Messages page keeps the send icon's count itself, as on web.
 */
export function startBell(onMessages: () => boolean): () => void {
  const refresh = async (): Promise<void> => {
    const [items, unread] = await Promise.allSettled([
      myNotifications(bell.notificationsOff.categories),
      onMessages() ? null : unreadMessageCount(),
    ]);
    // A failed round keeps the old count; the next round tries again.
    if (items.status === 'fulfilled') bell.showNotifications(items.value.items);
    if (unread.status === 'fulfilled' && unread.value !== null) messageBadge.unread = unread.value;
  };
  void refresh();
  const stop = startVisibilityPoll(refresh, { activeMs: 30_000 });
  return () => {
    stop();
    bell.showNotifications([]);
    messageBadge.unread = 0;
  };
}
