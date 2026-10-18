import type { Avatar, Person, Profile, Standing, StandingLevel } from '#lib/domain/people.ts';
import {
  callZome,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

const identity = <T>(fn: string, payload?: unknown) =>
  callZome<T>('directory', 'identity', fn, payload);

interface StandingWire {
  level: StandingLevel;
  founder: boolean;
  grant: ActionHash | null;
  vouchers: AgentPubKey[];
}

export async function myAgent(): Promise<string> {
  return hashToString(await identity<AgentPubKey>('my_agent'));
}

export async function getPerson(agent: string): Promise<Person> {
  const key = stringToHash(agent);
  const [profile, standing] = await Promise.all([
    identity<Profile | null>('get_profile', key),
    identity<StandingWire>('get_standing', key),
  ]);
  return { agent, profile, standing: toStanding(standing) };
}

export async function listPeople(): Promise<string[]> {
  return (await identity<AgentPubKey[]>('list_people')).map(hashToString);
}

export interface ProfileInput {
  name: string;
  bio: string;
  home_area: string;
  avatar: Avatar | null;
}

export async function saveProfile(input: ProfileInput): Promise<void> {
  await identity<ActionHash>('set_profile', input);
}

export async function vouchFor(agent: string): Promise<void> {
  await identity<ActionHash>('vouch', stringToHash(agent));
}

/** Turn received vouches (or steady activity) into Full standing. */
export async function claimFullStanding(): Promise<void> {
  await identity<ActionHash>('claim_full_standing');
}

export async function founders(): Promise<{ name: string; agent: string }[]> {
  return identity('founders');
}

function toStanding(wire: StandingWire): Standing {
  return { level: wire.level, founder: wire.founder, vouchers: wire.vouchers.map(hashToString) };
}
