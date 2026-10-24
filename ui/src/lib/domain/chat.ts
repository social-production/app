/** Mirrors the project DNA's discussion zome (ADR 0038). Times are Holochain micros. */

/** sp_core::chat::MESSAGE_MAX. */
export const MESSAGE_MAX = 4_000;

/** A photo or file on a direct or group message (web's MessageAttachment, ADR 0049). */
export interface ChatAttachment {
  kind: 'image' | 'file';
  name: string;
  mime: string;
  size: number;
  /** The parts holding its bytes, in order; empty on a copy still being sent. */
  chunks: string[];
  /** A copy being sent shows the file picked on this device. */
  localUrl?: string;
}

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
  /** Direct and group chats only: a leave shows as a system line, a finished call as a call
   * line (direct chats, ADR 0051), edits and pins as marks. */
  kind?: ChatKindOfLine;
  edited?: boolean;
  pinned?: boolean;
  attachments?: ChatAttachment[];
  call?: CallLine | null;
}

export type ChatKindOfLine = 'text' | 'system' | 'call';

/** web's `callOutcome` and `callDurationSeconds`. */
export interface CallLine {
  outcome: 'completed' | 'missed' | 'declined';
  seconds: number;
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
  /** `system`: "<author> left the group."; `call`: web's "Voice call · 4:12" line. */
  kind: ChatKindOfLine;
  edited: boolean;
  pinned: boolean;
  attachments: ChatAttachment[];
  call: CallLine | null;
  /** Your own message in a direct chat: web's ✓ (sent) or ✓✓ (the other person read it). */
  receipt?: 'sent' | 'read';
}

/** web's mention pattern (utils/linkifyMessageBody.ts, web-backend governance.py). */
export const MENTION = /(^|[^\w])@([A-Za-z0-9_-]{1,32})/g;

/** The usernames a message mentions, lowercased (web-backend `_mentioned_user_ids`). */
export function mentionsOf(body: string): string[] {
  return [...new Set([...body.matchAll(MENTION)].map((m) => (m[2] ?? '').toLowerCase()))];
}
