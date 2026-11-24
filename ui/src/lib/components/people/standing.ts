import { t } from '#lib/i18n/index.ts';
import type { Person } from '#lib/domain/people.ts';

/** The standing chip text, in web's words. */
export function standingLabel(person: Person): string {
  if (person.standing.founder) return t('Founder');
  if (person.standing.level === 'Full') return t('Full member');
  if (person.standing.level === 'Paused') return t('Paused');
  return t('New member');
}

export function standingHint(person: Person): string {
  // Everyone votes (ADR 0118); vouching and bot marks are what full standing adds.
  if (person.standing.level === 'Full') return t('Can vouch for people and mark bots.');
  if (person.standing.level === 'Paused') return t('Paused after a bot review.');
  return t('{n} of 2 vouches; then they can vouch and mark bots.', {
    n: person.standing.vouchers.length,
  });
}
