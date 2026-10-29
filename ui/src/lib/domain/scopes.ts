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
