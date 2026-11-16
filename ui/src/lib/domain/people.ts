/** Mirrors the directory DNA's identity zome. Agent keys cross the UI as base64 strings. */

export type StandingLevel = 'Paused' | 'New' | 'Full';

export interface Avatar {
  mime: string;
  bytes: Uint8Array;
}

export interface Profile {
  name: string;
  bio: string;
  home_area: string;
  avatar: Avatar | null;
}

export interface Standing {
  level: StandingLevel;
  founder: boolean;
  /** Distinct people who vouched for this person. */
  vouchers: string[];
}

export interface Person {
  agent: string;
  profile: Profile | null;
  standing: Standing;
}

// Limits mirror sp_core::limits (validation enforces them; the UI just helps).
export const BIO_MAX = 1000;
export const HOME_AREA_MAX = 100;
export const AVATAR_MAX_BYTES = 256 * 1024;
/** `identity_integrity::COVER_MAX_BYTES` (ADR 0107). */
export const COVER_MAX_BYTES = 512 * 1024;
export const VOUCHES_FOR_FULL = 2;

/** Short, stable tag that tells two people with the same name apart: "Johan · 7NFm". */
export function keyTag(agent: string): string {
  return agent.slice(5, 9);
}

/** Someone's name where you read it: just the name (Detail Pages Cleanup §5; there are no
 * @usernames, and pickers add the key tag only when two names clash). */
export function displayName(person: Pick<Person, 'profile'>): string {
  return person.profile?.name || 'Unnamed';
}

/** A name with its 4-character key tag, where every row has the same name (your devices). */
export function taggedName(person: Pick<Person, 'agent' | 'profile'>): string {
  return `${displayName(person)} · ${keyTag(person.agent)}`;
}

/** "Unnamed, Priya and 2 more": the first two first names of `names`, then how many more
 * there are out of `total` (a detail page's people row). */
export function namesLine(names: string[], total = names.length): string {
  const first = names.slice(0, 2).map((n) => n.trim().split(/\s+/)[0] || 'Unnamed');
  const more = total - first.length;
  if (more <= 0) return first.length === 2 ? `${first[0]} and ${first[1]}` : (first[0] ?? '');
  return `${first.join(', ')} and ${more} more`;
}
