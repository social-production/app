import { getContext, setContext } from 'svelte';

import type { Avatar } from '#lib/domain/people.ts';

/**
 * People's photos by key, for every face drawn from a name (owner, 2026-11-23: "profile
 * picture not displaying on comments, in feed etc."). Components stay presentational
 * (CODE_RULES), so the shell provides the lookup: `undefined` while it loads, `null` for none.
 */
export interface AvatarSource {
  of: (agent: string) => Avatar | null | undefined;
}

const KEY = Symbol('avatar-source');

export function provideAvatarSource(source: AvatarSource): void {
  setContext(KEY, source);
}

/** None outside the shell: faces then show initials. */
export function avatarSource(): AvatarSource | undefined {
  return getContext<AvatarSource | undefined>(KEY);
}
