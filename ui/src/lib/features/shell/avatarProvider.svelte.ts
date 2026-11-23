import { provideAvatarSource } from '#lib/components/people/avatarSource.ts';
import type { Avatar } from '#lib/domain/people.ts';
import { cachedPerson } from '#lib/services/people.ts';

/** Photos by key, read once per person (`cachedPerson`) and shared by every face. */
export function provideAvatars(): void {
  const photos = $state<Record<string, Avatar | null>>({});
  // Plain on purpose: asking is bookkeeping, not something a face redraws for.
  const asked: Record<string, true | undefined> = {};
  provideAvatarSource({
    of(agent) {
      if (!asked[agent]) {
        asked[agent] = true;
        // Not while the face is being drawn: state can't change during a render.
        queueMicrotask(() => {
          cachedPerson(agent)
            .then((p) => (photos[agent] = p.profile?.avatar ?? null))
            .catch(() => (asked[agent] = undefined));
        });
      }
      return photos[agent];
    },
  });
}
