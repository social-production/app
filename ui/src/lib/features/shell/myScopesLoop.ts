import type { ResolvedPathname } from '$app/types';

import { resolve } from '$app/paths';

import { myScopes } from '#lib/services/scopes.ts';
import { showMyScopes } from '#lib/stores/myScopes.svelte.ts';

import { startVisibilityPoll } from '#lib/components/actions/visibilityPoll.ts';
import type { Scope } from '#lib/domain/scopes.ts';

/** Keeps the left rail's channels and communities fresh while signed in (every minute). */
export function startMyScopes(): () => void {
  const refresh = async (): Promise<void> => {
    try {
      showMyScopes(await myScopes());
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

/** A scope's page. */
export const scopeHref = (s: Scope): ResolvedPathname =>
  s.kind === 'channel'
    ? resolve('/channels/[slug]', { slug: s.slug })
    : resolve('/communities/[slug]', { slug: s.slug });

/** The left rail's links to the scopes you joined. */
export const railLinks = (scopes: Scope[]): { href: ResolvedPathname; label: string }[] =>
  scopes.map((s) => ({ href: scopeHref(s), label: s.handle }));
