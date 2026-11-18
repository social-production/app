import { feedRules, postHidersIn, postShown } from '#lib/domain/privacy.ts';
import type { ProjectCard } from '#lib/domain/projects.ts';

import { followCircles, privacyOf } from './follows.ts';
import { myAgent } from './people.ts';

/**
 * Leaves out posts whose authors show them to followers only (ADR 0112), unless you follow
 * them. Lists that mix in strangers' posts (Search) pass their cards through here.
 */
export async function visiblePosts(cards: ProjectCard[]): Promise<ProjectCard[]> {
  const authors = cards.filter((c) => c.mode === 'post').map((c) => c.creator);
  if (!authors.length) return cards;
  const [found, circles, me] = await Promise.all([
    privacyOf(authors),
    followCircles(),
    myAgent().catch(() => ''),
  ]);
  const hiders = postHidersIn(found);
  if (!hiders.length) return cards;
  const rules = feedRules([], [...circles.following, ...(me ? [me] : [])], hiders);
  return cards.filter((c) => c.mode !== 'post' || postShown(rules, c.creator));
}
