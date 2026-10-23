/**
 * Who "@na…" suggests and who "@name" means (ADR 0047). web-backend ranks people you follow,
 * then your followers, then everyone (people_suggestions.py). The app has no follows yet, so
 * the nearest circles are the people in this chat, then the people you have direct or group
 * chats with; follows slot in between when they are ported. Within a circle an exact name
 * comes first, then names starting with the query, then names containing it, by name.
 */
export interface Mentionable {
  agent: string;
  name: string;
}

export const MENTION_RESULTS = 8;

/** Nearest circle first; anyone in none of them comes last. */
function circleOf(agent: string, circles: readonly string[][]): number {
  const i = circles.findIndex((c) => c.includes(agent));
  return i === -1 ? circles.length : i;
}

function matchOf(name: string, needle: string): number {
  const lower = name.toLowerCase();
  if (lower === needle) return 0;
  if (lower.startsWith(needle)) return 1;
  return lower.includes(needle) ? 2 : -1;
}

/** People whose name contains `query`, nearest first; never `me`. */
export function rankMentions(
  query: string,
  people: Mentionable[],
  circles: readonly string[][],
  me: string,
  limit = MENTION_RESULTS,
): Mentionable[] {
  const needle = query.trim().toLowerCase();
  const seen = new Set<string>([me]);
  return people
    .filter((p) => !seen.has(p.agent) && seen.add(p.agent))
    .map((p) => ({ p, circle: circleOf(p.agent, circles), match: matchOf(p.name, needle) }))
    .filter((r) => r.match !== -1)
    .sort(
      (a, b) =>
        a.circle - b.circle ||
        a.match - b.match ||
        a.p.name.localeCompare(b.p.name, undefined, { sensitivity: 'base' }),
    )
    .slice(0, limit)
    .map((r) => r.p);
}

/** The person "@name" most likely means here (names aren't unique), or `null`. */
export function mentionTarget(
  name: string,
  people: Mentionable[],
  circles: readonly string[][],
): string | null {
  const lower = name.toLowerCase();
  const named = people.filter((p) => p.name.toLowerCase() === lower);
  return rankMentions(name, named, circles, '', 1)[0]?.agent ?? null;
}
