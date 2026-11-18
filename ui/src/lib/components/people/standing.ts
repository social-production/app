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
  if (person.standing.level === 'Full') return t('Votes and vouches count fully.');
  if (person.standing.level === 'Paused') return t('Paused after a bot review.');
  return `${person.standing.vouchers.length}/2 vouches from full members`;
}
