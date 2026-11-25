import { t } from '#lib/i18n/index.ts';
import type { RailItem, RailVoteKind } from './activityRail.ts';
import { isHelpKind } from './helpRail.ts';
import { kindOfSpace } from './kindGlyphs.ts';

/** What a rail card is, above its title: the vote's kind, or whose activity it is. */
const SOFTWARE_KICKERS: Partial<Record<RailVoteKind, string>> = {
  pull_request_merge: 'Action · merge needed',
  pull_request: 'Vote · pull request',
  merge_capability: 'Vote · merge capability',
  repository_replacement: 'Vote · repository',
};

/** web's `itemKicker`. */
export function railKicker(item: RailItem): string {
  if (item.kind === 'vote') {
    const software = item.voteKindLabel && SOFTWARE_KICKERS[item.voteKindLabel];
    if (software) return software;
    const kind = item.voteKindLabel ? ` · ${t(item.voteKindLabel.replace('_', ' '))}` : '';
    return `${t('Vote')}${kind}`;
  }
  if (item.kind === 'request') return t('Service request');
  if (isHelpKind(item.kind)) return t('Request');
  const kind = kindOfSpace(item.projectMode, item.subtype);
  if (kind === 'meetup') return t('Meetup activity');
  if (kind === 'event') return t('Event activity');
  return t('Project activity');
}
