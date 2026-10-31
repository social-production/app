import type { CharterMode } from '#lib/domain/projects.ts';

import { resolve } from '$app/paths';

/** A space's page by its mode: a project, an event (ADR 0057) a help request (ADR 0067), a thread or a post (ADR 0070). */
export function spaceHref(mode: CharterMode | null | undefined, id: string) {
  if (mode === 'event') return resolve('/events/[id]', { id });
  if (mode === 'help-request') return resolve('/help-requests/[id]', { id });
  if (mode === 'thread') return resolve('/threads/[id]', { id });
  if (mode === 'post') return resolve('/posts/[id]', { id });
  return resolve('/projects/[id]', { id });
}
