import {
  categoryOf,
  type NotificationCategory,
  type NotificationItem,
} from '#lib/domain/notifications.ts';

/**
 * The notifications list and the bell's unread count (web's unreadCounts.notifications). The
 * app shell refreshes it while someone is signed in; the Notifications page reads and marks it.
 * Categories switched off in Settings are left out (web's notification settings, per device).
 */
export const notifications = $state<{ items: NotificationItem[]; unread: number; invites: number }>(
  {
    items: [],
    unread: 0,
    // web's unreadCounts.invites: waiting invites, whatever the notification settings.
    invites: 0,
  },
);

const OFF_KEY = 'sp_notifications_off';

function readOff(): NotificationCategory[] {
  try {
    return JSON.parse(localStorage.getItem(OFF_KEY) ?? '[]') as NotificationCategory[];
  } catch {
    return [];
  }
}

/** Categories switched off on this device (web's defaults have all of these on). */
export const notificationsOff = $state<{ categories: NotificationCategory[] }>({
  categories: readOff(),
});

let latest: NotificationItem[] = [];

export function showNotifications(items: NotificationItem[]): void {
  latest = items;
  notifications.invites = items.filter((n) => n.kind === 'evt-invite').length;
  const shown = items.filter((n) => !notificationsOff.categories.includes(categoryOf[n.kind]));
  notifications.items = shown;
  notifications.unread = shown.filter((n) => n.isUnread).length;
}

export function setNotificationCategory(category: NotificationCategory, on: boolean): void {
  const rest = notificationsOff.categories.filter((c) => c !== category);
  notificationsOff.categories = on ? rest : [...rest, category];
  try {
    localStorage.setItem(OFF_KEY, JSON.stringify(notificationsOff.categories));
  } catch {
    // Private windows: the choice holds until the page closes.
  }
  showNotifications(latest);
}
