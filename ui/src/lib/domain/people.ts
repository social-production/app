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
export const VOUCHES_FOR_FULL = 2;

/** Short, stable tag that tells two people with the same name apart: "Johan · 7NFm". */
export function keyTag(agent: string): string {
  return agent.slice(5, 9);
}

export function displayName(person: Pick<Person, 'agent' | 'profile'>): string {
  return `${person.profile?.name ?? 'Unnamed'} · ${keyTag(person.agent)}`;
}
