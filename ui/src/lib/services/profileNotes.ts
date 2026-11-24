import type { CommentActivity } from '#lib/domain/commentNotes.ts';
import { excerptOf } from '#lib/domain/commentNotes.ts';
import type { Avatar } from '#lib/domain/people.ts';
import type { CharterMode } from '#lib/domain/projects.ts';
import {
  callZome,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { queued } from './chat.ts';
import { withCounts } from './commentCounts.ts';
import { namesOf } from './people.ts';

/** Profile covers and comment notes in the directory (ADR 0107). */
const identity = <T>(fn: string, payload?: unknown) =>
  callZome<T>('directory', 'identity', fn, payload);

/** Someone's cover photo, or null. */
export async function getCover(agent: string): Promise<Avatar | null> {
  return identity<Avatar | null>('get_cover', stringToHash(agent));
}

/** Replace your cover; null removes it. */
export async function setCover(cover: Avatar | null): Promise<void> {
  await identity<ActionHash | null>('set_cover', cover);
}

/**
 * Note a comment you just wrote in a space's chat, so profiles and followers' Home can list
 * it. Callers wait for it (leaving the page mid-call drops it). Best effort: the comment
 * stands if the note fails.
 */
export async function noteComment(
  space: string,
  message: string,
  body: string,
  replyTo: string | null,
): Promise<void> {
  const excerpt = excerptOf(body);
  if (!excerpt) return;
  // In the chain's write queue (a write racing another one on the same chain can fail).
  await queued(() =>
    identity<ActionHash>('note_comment', {
      space: stringToHash(space),
      message: stringToHash(message),
      reply_to: replyTo ? stringToHash(replyTo) : null,
      excerpt,
    }),
  ).catch(() => undefined);
}

/** Take back the note of a comment you deleted. */
export async function unnoteComment(message: string): Promise<void> {
  await queued(() => identity<null>('unnote_comment', stringToHash(message))).catch(
    () => undefined,
  );
}

interface NoteWire {
  id: ActionHash;
  author: AgentPubKey;
  created_at: number;
  note: { space: ActionHash; message: ActionHash; reply_to: AgentPubKey | null; excerpt: string };
}

interface ListingWire {
  listing: { title: string; mode: string; tags: string[] };
}

const listings = new Map<string, Promise<ListingWire | null>>();

/** A space's listing, read once per session (title and type rarely change). */
function listingOf(id: string): Promise<ListingWire | null> {
  let found = listings.get(id);
  if (!found) {
    found = identity<ListingWire | null>('get_listing', stringToHash(id)).catch(() => null);
    listings.set(id, found);
  }
  return found;
}

/** The newest comments of these people (at most 50 each), newest first, with the spaces they
 * were in. A space that is not listed (a private event) is left out. */
export async function commentsOf(agents: string[]): Promise<CommentActivity[]> {
  const read = await Promise.allSettled(
    agents.map((a) => identity<NoteWire[]>('comments_of', stringToHash(a))),
  );
  const notes = read.flatMap((r) => (r.status === 'fulfilled' ? r.value : []));
  if (!notes.length) return [];
  const people = notes.flatMap((n) => [n.author, ...(n.note.reply_to ? [n.note.reply_to] : [])]);
  const keys = [...new Set(people.map(hashToString))];
  const [names, spaces] = await Promise.all([
    namesOf(keys),
    Promise.all(notes.map((n) => listingOf(hashToString(n.note.space)))),
  ]);
  const out: CommentActivity[] = [];
  notes.forEach((n, i) => {
    const space = spaces[i];
    if (!space) return;
    const author = hashToString(n.author);
    out.push({
      id: hashToString(n.id),
      author,
      authorName: names[author] ?? 'Unnamed',
      createdAt: n.created_at,
      excerpt: n.note.excerpt,
      message: hashToString(n.note.message),
      replyToName: n.note.reply_to ? (names[hashToString(n.note.reply_to)] ?? 'Unnamed') : null,
      space: {
        id: hashToString(n.note.space),
        title: space.listing.title,
        mode: space.listing.mode as CharterMode,
        tag: space.listing.tags[0] ?? '',
      },
    });
  });
  return withCounts(out.sort((a, b) => b.createdAt - a.createdAt));
}

/** A quoted space's title, type and first tag (quote posts, Owner Notes Mockups §6). */
export async function quotedSpace(
  id: string,
): Promise<{ title: string; mode: CharterMode; tag: string | null } | null> {
  const found = await listingOf(id);
  if (!found) return null;
  const { title, mode, tags } = found.listing;
  return { title, mode: mode as CharterMode, tag: tags[0] ?? null };
}
