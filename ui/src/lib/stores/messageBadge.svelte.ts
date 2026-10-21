/**
 * The send icon's unread count (web's unreadCounts.messages): unread messages in your shown
 * personal chats plus your project chats. The app shell refreshes it while someone is signed
 * in; the Messages page sets it from its own lists, so reading a chat clears it at once.
 */
export const messageBadge = $state<{ unread: number }>({ unread: 0 });
