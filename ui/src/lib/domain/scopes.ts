/** Channels and communities (ADR 0068), with web's handle rules (web/src/lib/utils/handles.ts). */

export type ScopeKind = 'channel' | 'community';

export interface Scope {
  /** The listing id. */
  id: string;
  kind: ScopeKind;
  /** The handle as entered (the charter's title); `slug` is how it's compared and linked. */
  handle: string;
  slug: string;
  description: string;
  members: number;
  creator: string;
  createdAt: number;
  networkSeed: string;
  /** A closed (private) community: its own network, found from your memberships, never
   * listed; its id is its seed (ADR 0068 decision 1). */
  isPrivate?: boolean;
}

/** web's canonicalizeHandle. */
export const slugOf = (handle: string): string => handle.trim().toLowerCase();

/** The tag an item carries for a scope: `#slug` (channel) or `@slug` (community). */
export const scopeTag = (kind: ScopeKind, slug: string): string =>
  `${kind === 'channel' ? '#' : '@'}${slug}`;

/** A tag's scope, if it is one. */
export function tagScope(tag: string): { kind: ScopeKind; slug: string } | null {
  if (tag.startsWith('#')) return { kind: 'channel', slug: tag.slice(1) };
  if (tag.startsWith('@')) return { kind: 'community', slug: tag.slice(1) };
  return null;
}

const RESERVED = ['platform', 'stewardship'];

/** web's validateHandle: 3–32 letters, numbers, hyphens or underscores, single separators. */
export function checkHandle(
  handle: string,
  label: string,
  kind: ScopeKind,
): { ok: true; slug: string } | { ok: false; error: string } {
  const display = handle.trim();
  if (!display) return { ok: false, error: `${label} is required` };
  if (/\s/.test(display)) return { ok: false, error: `${label} cannot contain spaces` };
  if (display.length < 3 || display.length > 32)
    return { ok: false, error: `${label} must be 3–32 characters` };
  if (!/^[A-Za-z0-9]+(?:[-_][A-Za-z0-9]+)*$/.test(display))
    return {
      ok: false,
      error:
        'Use 3–32 characters: letters, numbers, hyphens, or underscores. No spaces, and no leading, trailing, or repeated separators.',
    };
  if (kind === 'channel' && RESERVED.includes(slugOf(display)))
    return { ok: false, error: 'That channel name is reserved' };
  return { ok: true, slug: slugOf(display) };
}

/** web's TagRef for a card's tag: a scope (icon + its page) or a free word (search). */
export interface TagRef {
  tag: string;
  kind: ScopeKind | 'free';
  slug: string;
  label: string;
}

export function tagRef(tag: string): TagRef {
  const scope = tagScope(tag);
  return scope
    ? { tag, ...scope, label: scope.slug }
    : { tag, kind: 'free', slug: tag, label: tag };
}

/** A closed community's invite link (ADR 0068 decision 5): the seed rides in the fragment,
 * which browsers never send to a server. */
export const inviteLink = (origin: string, slug: string, seed: string): string =>
  `${origin}/communities/${encodeURIComponent(slug)}#invite=${encodeURIComponent(seed)}`;

const PRIVATE_SEED = /^[^.\s#&]+\.(private|organized)\.[^.\s#&]+$/;

/** web's parseInviteToken: the seed from a pasted invite link, or the code itself. */
export function parseInvite(raw: string): string | null {
  const value = raw.trim();
  const match = value.match(/[#?&]invite=([^&#\s]+)/);
  let seed = match?.[1] ?? value;
  try {
    seed = decodeURIComponent(seed);
  } catch {
    return null;
  }
  return PRIVATE_SEED.test(seed) ? seed : null;
}
