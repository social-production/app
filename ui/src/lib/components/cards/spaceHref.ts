import type { CharterMode } from '#lib/domain/projects.ts';

import { resolve } from '$app/paths';

/** A space's page by its mode: a project, an event (ADR 0057) or a help request (ADR 0067). */
export function spaceHref(mode: CharterMode | null | undefined, id: string) {
  if (mode === 'event') return resolve('/events/[id]', { id });
  if (mode === 'help-request') return resolve('/help-requests/[id]', { id });
  return resolve('/projects/[id]', { id });
}
