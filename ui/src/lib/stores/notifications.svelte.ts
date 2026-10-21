import type { NotificationItem } from '#lib/domain/notifications.ts';

/**
 * The notifications list and the bell's unread count (web's unreadCounts.notifications). The
 * app shell refreshes it while someone is signed in; the Notifications page reads and marks it.
 */
export const notifications = $state<{ items: NotificationItem[]; unread: number }>({
  items: [],
  unread: 0,
});

export function showNotifications(items: NotificationItem[]): void {
  notifications.items = items;
  notifications.unread = items.filter((n) => n.isUnread).length;
}
