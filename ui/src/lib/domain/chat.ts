/** Mirrors the project DNA's discussion zome (ADR 0038). Times are Holochain micros. */

/** sp_core::chat::MESSAGE_MAX. */
export const MESSAGE_MAX = 4_000;

/** The original a reply quotes. */
export interface ChatQuote {
  id: string;
  /** `null` while the original can't be reached. */
  author: string | null;
  preview: string;
  deleted: boolean;
}

export interface ChatMessage {
  id: string;
  author: string;
  body: string;
  createdAt: number;
  replyTo: ChatQuote | null;
  /** Direct and group chats only: a leave shows as a system line, edits and pins as marks. */
  kind?: 'text' | 'system';
  edited?: boolean;
  pinned?: boolean;
}

/** One message as the chat panel shows it (web's LiveChatPanel `ChatMessage`). */
export interface ChatLine {
  id: string;
  author: string;
  authorName: string;
  body: string;
  /** Milliseconds. */
  createdAt: number;
  own: boolean;
  /** Being sent: not on the network yet. */
  pending: boolean;
  replyToId: string | null;
  replyAuthor: string | null;
  replyPreview: string | null;
  /** `system`: "<author> left the group." */
  kind: 'text' | 'system';
  edited: boolean;
  pinned: boolean;
  /** Your own message in a direct chat: web's ✓ (sent) or ✓✓ (the other person read it). */
  receipt?: 'sent' | 'read';
}

/** web's mention pattern (utils/linkifyMessageBody.ts, web-backend governance.py). */
export const MENTION = /(^|[^\w])@([A-Za-z0-9_-]{1,32})/g;

/** The usernames a message mentions, lowercased (web-backend `_mentioned_user_ids`). */
export function mentionsOf(body: string): string[] {
  return [...new Set([...body.matchAll(MENTION)].map((m) => (m[2] ?? '').toLowerCase()))];
}
