import type { ChatLine } from '#lib/domain/chat.ts';

/** Display helpers ported from web's LiveChatPanel and utils/linkifyMessageBody. */

/** Clock time in the bubble; the day goes in a divider above, as in other messaging apps. */
export function formatMessageTime(ms: number): string {
  return new Date(ms).toLocaleTimeString(undefined, { hour: 'numeric', minute: '2-digit' });
}

export function messageDayKey(ms: number): string {
  return new Date(ms).toDateString();
}

export function messageDayLabel(ms: number): string {
  const date = new Date(ms);
  const today = new Date();
  const yesterday = new Date();
  yesterday.setDate(today.getDate() - 1);
  if (date.toDateString() === today.toDateString()) return 'Today';
  if (date.toDateString() === yesterday.toDateString()) return 'Yesterday';
  const sameYear = date.getFullYear() === today.getFullYear();
  return date.toLocaleDateString(undefined, {
    weekday: 'short',
    day: 'numeric',
    month: 'short',
    ...(sameYear ? {} : { year: 'numeric' }),
  });
}

/** Name once per run of messages from the same person, like a group chat elsewhere. */
export function showsAuthor(message: ChatLine, previous: ChatLine | undefined): boolean {
  if (!previous || previous.author !== message.author) return true;
  if (messageDayKey(previous.createdAt) !== messageDayKey(message.createdAt)) return true;
  return message.createdAt - previous.createdAt > 5 * 60_000;
}

export type BodyPart = { text: string; href?: undefined } | { text: string; href: string };

const LINK = /(https?:\/\/[^\s<]+|www\.[^\s<]+)/gi;
const TRAILING = /[.,;:!?'")\]}]+$/;

/**
 * web linkifies bare URLs with `{@html}`; here the body is split into text and link parts, so
 * nothing a person writes is ever parsed as HTML.
 */
export function bodyParts(body: string): BodyPart[] {
  const parts: BodyPart[] = [];
  let last = 0;
  for (const match of body.matchAll(LINK)) {
    const raw = match[0];
    const trailing = TRAILING.exec(raw)?.[0] ?? '';
    const core = raw.slice(0, raw.length - trailing.length);
    if (!core) continue;
    if (match.index > last) parts.push({ text: body.slice(last, match.index) });
    const href = core.toLowerCase().startsWith('www.') ? `https://${core}` : core;
    parts.push({ text: core, href });
    last = match.index + core.length;
  }
  if (last < body.length) parts.push({ text: body.slice(last) });
  return parts;
}
