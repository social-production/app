import type { Scope } from '#lib/domain/scopes.ts';

/** The channels and communities the viewer joined, for the left rail (web's directory lists). */
export const myScopes = $state<{ channels: Scope[]; communities: Scope[] }>({
  channels: [],
  communities: [],
});

export function showMyScopes(scopes: Scope[]): void {
  const byHandle = (a: Scope, b: Scope) => a.slug.localeCompare(b.slug);
  myScopes.channels = scopes.filter((s) => s.kind === 'channel').sort(byHandle);
  myScopes.communities = scopes.filter((s) => s.kind === 'community').sort(byHandle);
}
