import type { Person } from '#lib/domain/people.ts';
import type { Pairing } from '#lib/services/people.ts';

export interface PersonRow {
  person: Person;
  /** Number of devices this person uses (1 when unpaired). */
  devices: number;
}

/**
 * One row per person: a paired device whose main account is also listed is folded into
 * that account's row (ADR 0024: "so people see one person").
 */
export function personRows(entries: { person: Person; pairing: Pairing }[]): PersonRow[] {
  const listed = new Set(entries.map((e) => e.person.agent));
  return entries
    .filter((e) => !(e.pairing.primary && listed.has(e.pairing.primary)))
    .map((e) => ({ person: e.person, devices: 1 + e.pairing.devices.length }));
}
