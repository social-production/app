import type { ResolvedPathname } from '$app/types';

import { resolve } from '$app/paths';

import { scopeImageUrl } from '#lib/services/scopeImages.ts';
import { myScopes } from '#lib/services/scopes.ts';
import { showMyScopes } from '#lib/stores/myScopes.svelte.ts';

import { startVisibilityPoll } from '#lib/components/actions/visibilityPoll.ts';
import type { Scope } from '#lib/domain/scopes.ts';

/** Keeps the left rail's channels and communities fresh while signed in (every minute). */
export function startMyScopes(): () => void {
  const refresh = async (): Promise<void> => {
    try {
      showMyScopes(await withPictures(await myScopes()));
    } catch {
      // keep the last lists; the next round tries again
    }
  };
  void refresh();
  const stop = startVisibilityPoll(refresh, { activeMs: 60_000 });
  return () => {
    stop();
    showMyScopes([]);
  };
}

/** Each scope with its picture's URL, when it has one (ADR 0109). */
export async function withPictures(scopes: Scope[]): Promise<Scope[]> {
  return Promise.all(
    scopes.map(async (s) => (s.picture ? { ...s, pictureUrl: await scopeImageUrl(s.picture) } : s)),
  );
}

/** A scope's page. */
export const scopeHref = (s: Scope): ResolvedPathname =>
  s.kind === 'channel'
    ? resolve('/channels/[slug]', { slug: s.slug })
    : resolve('/communities/[slug]', { slug: s.slug });

/** The left rail's links to the scopes you joined. */
export const railLinks = (
  scopes: Scope[],
): { href: ResolvedPathname; label: string; imageUrl: string | null }[] =>
  scopes.map((s) => ({ href: scopeHref(s), label: s.handle, imageUrl: s.pictureUrl ?? null }));
