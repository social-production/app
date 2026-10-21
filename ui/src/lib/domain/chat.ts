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
}
