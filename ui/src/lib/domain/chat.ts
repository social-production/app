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
}
