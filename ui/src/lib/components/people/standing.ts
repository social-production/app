import type { Person } from '#lib/domain/people.ts';

/** The standing chip text, in web's words. */
export function standingLabel(person: Person): string {
  if (person.standing.founder) return 'Founder';
  if (person.standing.level === 'Full') return 'Full member';
  if (person.standing.level === 'Paused') return 'Paused';
  return 'New member';
}

export function standingHint(person: Person): string {
  if (person.standing.level === 'Full') return 'Votes and vouches count fully.';
  if (person.standing.level === 'Paused') return 'Paused after a bot review.';
  return `${person.standing.vouchers.length}/2 vouches from full members`;
}
