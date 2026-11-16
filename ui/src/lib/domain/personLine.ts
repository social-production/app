import { keyTag, type Person, type StandingLevel } from './people.ts';

const STANDING: Record<StandingLevel, string> = {
  Full: 'Full member',
  New: 'New member',
  Paused: 'Paused',
};

/** Where someone picks a person (Detail Pages Cleanup §5): "Fitzroy · Full member". */
export function placeAndStanding(person: Pick<Person, 'profile' | 'standing'>): string {
  const place = person.profile?.home_area.trim() ?? '';
  const standing = STANDING[person.standing.level];
  return place ? `${place} · ${standing}` : standing;
}

/** The names that appear more than once in a list, lower-cased: only those show a key tag. */
export function clashingNames(people: { name: string }[]): Set<string> {
  const seen = new Set<string>();
  const twice = new Set<string>();
  for (const { name } of people) {
    const key = name.trim().toLowerCase();
    if (seen.has(key)) twice.add(key);
    seen.add(key);
  }
  return twice;
}

/** A picker's name: just the name, with the 4-character tag only when it clashes. */
export function pickerName(person: { agent: string; name: string }, clashes: Set<string>): string {
  return clashes.has(person.name.trim().toLowerCase())
    ? `${person.name} · ${keyTag(person.agent)}`
    : person.name;
}
