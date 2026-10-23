import { MENTION, type ChatLine } from '#lib/domain/chat.ts';

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

/** web's `messageIsRead`: in a direct chat, your own messages show ✓, or ✓✓ once the other
 * person has read up to them (`peerReadAt`, milliseconds). */
export function withReceipts(lines: ChatLine[], peerReadAt: number | null): ChatLine[] {
  return lines.map((l) =>
    l.own && !l.pending && l.kind === 'text'
      ? { ...l, receipt: peerReadAt !== null && peerReadAt >= l.createdAt ? 'read' : 'sent' }
      : l,
  );
}

/** Name once per run of messages from the same person, like a group chat elsewhere. */
export function showsAuthor(message: ChatLine, previous: ChatLine | undefined): boolean {
  if (!previous || previous.author !== message.author || previous.kind === 'system') return true;
  if (messageDayKey(previous.createdAt) !== messageDayKey(message.createdAt)) return true;
  return message.createdAt - previous.createdAt > 5 * 60_000;
}

/** Text, a link (`internal`: one of this app's own pages, opened in the same tab), or an
 * "@name" mention (web's `a.mention`). */
export type BodyPart =
  | { text: string; href?: undefined; internal?: undefined; mention?: undefined }
  | { text: string; href: string; internal: boolean; mention?: undefined }
  | { text: string; href?: undefined; internal?: undefined; mention: string };

/** web's internal paths, narrowed to the pages this app has (web's events, threads and posts
 * aren't ported, so their paths stay text). */
const PAGES = '(?:projects|messages|profile)';
const LINK = new RegExp(
  `(https?:\\/\\/[^\\s<]+|www\\.[^\\s<]+|\\/${PAGES}(?:\\/[^\\s<]+)?(?:\\?[^\\s<]+)?)`,
  'gi',
);
const TRAILING = /[.,;:!?'")\]}]+$/;
/** web's LABELLED_INTERNAL_LINE: "View request: /projects/x?…" on its own line reads as a
 * "View request" link, not a raw path. */
const LABELLED = new RegExp(`^([A-Za-z][A-Za-z ]{1,40}): (\\/${PAGES}\\S*)$`, 'gm');
/** web's withoutRawPaths' INTERNAL_PATH (all of web's pages, since previews only drop them). */
const RAW_PATH =
  /\/(?:projects|events|threads|messages|posts|profile|help-requests|communities|channels|invites)\/?\S*/;

/**
 * web linkifies bare URLs, internal paths and mentions with `{@html}`; here the body is split
 * into text, link and mention parts, so nothing a person writes is ever parsed as HTML.
 */
export function bodyParts(body: string): BodyPart[] {
  const parts: BodyPart[] = [];
  let last = 0;
  for (const match of body.matchAll(LABELLED)) {
    const [whole, label = '', path = ''] = match;
    if (match.index > last) parts.push(...linkParts(body.slice(last, match.index)));
    parts.push({ text: label, href: path, internal: true });
    last = match.index + whole.length;
  }
  if (last < body.length) parts.push(...linkParts(body.slice(last)));
  return parts;
}

/** web's linkBareUrls, then mentions between the links. */
function linkParts(text: string): BodyPart[] {
  const parts: BodyPart[] = [];
  let last = 0;
  for (const match of text.matchAll(LINK)) {
    const raw = match[0];
    const trailing = TRAILING.exec(raw)?.[0] ?? '';
    const core = raw.slice(0, raw.length - trailing.length);
    if (!core) continue;
    if (match.index > last) parts.push(...mentionParts(text.slice(last, match.index)));
    const internal = core.startsWith('/');
    const href = !internal && core.toLowerCase().startsWith('www.') ? `https://${core}` : core;
    parts.push({ text: core, href, internal });
    last = match.index + core.length;
  }
  if (last < text.length) parts.push(...mentionParts(text.slice(last)));
  return parts;
}

/**
 * web's withoutRawPaths, for chat-list previews: "View request: /projects/x?…" becomes
 * "View request", and a stray "Open /projects/x" loses the path.
 */
export function withoutRawPaths(text: string): string {
  return text
    .replace(LABELLED, (_line, label: string) => label)
    .replace(new RegExp(`\\s*\\bOpen ${RAW_PATH.source}`, 'g'), '')
    .replace(new RegExp(`\\s*${RAW_PATH.source}`, 'g'), '')
    .trim();
}
/** web's linkMentions, outside links only. */
function mentionParts(text: string): BodyPart[] {
  const parts: BodyPart[] = [];
  let last = 0;
  for (const match of text.matchAll(MENTION)) {
    const [whole, prefix = '', name = ''] = match;
    const at = match.index + prefix.length;
    if (at > last) parts.push({ text: text.slice(last, at) });
    parts.push({ text: `@${name}`, mention: name });
    last = match.index + whole.length;
  }
  if (last < text.length) parts.push({ text: text.slice(last) });
  return parts;
}

/** web LiveChatPanel's formatByteSize, for file bubbles. */
export function formatByteSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.max(1, Math.round(bytes / 1024))} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
