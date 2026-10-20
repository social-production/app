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
  /** Who manages a group now (web's `createdBy`, which moves when the manager leaves). */
  manager: string | null;
  /** Everyone in the chat now, longest-standing first. */
  members: string[];
  title: string | null;
  /** `system` is a leave: "<name> left the group." */
  last: { author: string; preview: string; createdAt: number; kind: 'text' | 'system' } | null;
  unread: number;
  /** When the invitation arrived (orders chats with no messages yet). */
  invitedAt: number;
  /** You're still in it (not removed, not left). */
  member: boolean;
  /** web's `can_pin`: either person in a direct chat, the manager in a group. */
  canPin: boolean;
  /** Your own list settings (web's chat menu). */
  pinned: boolean;
  muted: boolean;
  hidden: boolean;
}

/** One pinned message, as web's pin bar shows it. */
export interface ChatPin {
  messageId: string;
  preview: string;
}

/** web's chat menu settings; only the given ones change. */
export interface ListPrefs {
  pinned?: boolean;
  muted?: boolean;
  hidden?: boolean;
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

/** A project chat as web's Public tab lists it (web's MessageLinkedChat, projects only). */
export interface PublicChat {
  /** The project's network seed (the list knows the clones on this node, not their listings). */
  id: string;
  title: string;
  members: number;
  last: { author: string; preview: string; createdAt: number } | null;
  unread: number;
  /** The last message, else the project's last activity (orders the list). */
  activeAt: number;
  /** Your own list settings, kept on this device (ADR 0044). */
  pinned: boolean;
  muted: boolean;
}
