import type { VoteDirection } from './discussions.ts';
import type { CharterMode } from './projects.ts';

/** `NOTE_MAX`: a comment note keeps the comment's first 280 characters (ADR 0107). */
export const EXCERPT_MAX = 280;

/** A comment someone wrote, as a feed row shows it (Profiles and Comments mockups §3). */
export interface CommentActivity {
  /** The note's id (the row's key). */
  id: string;
  author: string;
  authorName: string;
  /** Microseconds, as feed cards. */
  createdAt: number;
  excerpt: string;
  /** The message in the space's chat (the row opens it). */
  message: string;
  /** Whom a reply answers, by name. */
  replyToName: string | null;
  space: { id: string; title: string; mode: CharterMode; tag: string };
  /** A thread's or post's comment: its votes, replies and your vote (owner, 2026-11-24). */
  counts?: CommentCounts;
}

export interface CommentCounts {
  /** Up votes minus down votes. */
  score: number;
  replies: number;
  mine: VoteDirection;
}

/** The opening words a note keeps: one line of whitespace, cut on a word if it is long. */
export function excerptOf(body: string): string {
  const flat = body.replace(/\s+/g, ' ').trim();
  if (flat.length <= EXCERPT_MAX) return flat;
  const cut = flat.slice(0, EXCERPT_MAX - 1);
  const space = cut.lastIndexOf(' ');
  return `${space > EXCERPT_MAX / 2 ? cut.slice(0, space) : cut}…`;
}

/** Threads and posts hold comments; every other space holds a chat. */
export const inChat = (mode: CharterMode): boolean => mode !== 'thread' && mode !== 'post';

/**
 * The row's context line: t('Maya Kim commented'), "You replied to Priya R.", "Tomas B. commented
 * in the chat". `you` when the viewer wrote it.
 */
export function commentContext(c: CommentActivity, you: boolean): string {
  const who = you ? 'You' : c.authorName;
  if (c.replyToName) return `${who} replied to ${c.replyToName}`;
  return inChat(c.space.mode) ? `${who} commented in the chat` : `${who} commented`;
}

/** The type word on the quoted card. */
export function spaceKind(mode: CharterMode): string {
  const words: Partial<Record<CharterMode, string>> = {
    thread: 'Thread',
    post: 'Post',
    event: 'Event',
    'help-request': 'Request',
    proposal: 'Proposal',
    channel: 'Channel',
    community: 'Community',
  };
  return words[mode] ?? 'Project';
}
