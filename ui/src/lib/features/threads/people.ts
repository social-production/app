import { getPerson, myAgent } from '#lib/services/people.ts';

/** Names for the agents not yet known, merged into `known` (web shows usernames). */
export async function withNames(
  agents: string[],
  known: Record<string, string>,
): Promise<Record<string, string>> {
  const missing = [...new Set(agents)].filter((a) => !(a in known));
  if (missing.length === 0) return known;
  const people = await Promise.all(missing.map(getPerson));
  const names = { ...known };
  for (const p of people) names[p.agent] = p.profile?.name ?? 'Unnamed';
  return names;
}

/** The viewer's key, name and whether a community review paused the account. */
export async function viewer(): Promise<{ me: string; name: string; paused: boolean }> {
  const me = await myAgent();
  const person = await getPerson(me);
  return {
    me,
    name: person.profile?.name ?? 'Unnamed',
    paused: person.standing.level === 'Paused',
  };
}
