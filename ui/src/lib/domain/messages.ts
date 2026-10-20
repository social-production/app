/** Mirrors the conversation DNA's messages zome (ADR 0043). Times are Holochain micros. */

export type ChatKind = 'direct' | 'group';

/** sp_core::conversation limits. */
export const GROUP_TITLE_MAX = 200;
export const CHAT_MEMBERS_MAX = 50;

/** One direct or group chat as web's Personal list shows it. */
export interface ChatSummary {
  /** The chat's network seed: its id in the UI. */
  id: string;
  kind: ChatKind;
  creator: string;
  /** Everyone in the chat, the creator first. */
  members: string[];
  title: string | null;
  last: { author: string; preview: string; createdAt: number } | null;
  unread: number;
  /** When the invitation arrived (orders chats with no messages yet). */
  invitedAt: number;
}

/** Someone to message, as the compose sheet suggests them. */
export interface Contact {
  agent: string;
  name: string;
}

/** What the compose sheet asks to start. */
export interface NewChat {
  kind: ChatKind;
  members: Contact[];
  title: string;
  body: string;
}
