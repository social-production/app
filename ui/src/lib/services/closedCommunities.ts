import {
  isPrivateSeed,
  projectSubtype,
  summaryOf,
  type Project,
  type ProjectCard,
} from '#lib/domain/projects.ts';
import { slugOf, type Scope } from '#lib/domain/scopes.ts';
import {
  callCell,
  clonedSeeds,
  newPrivateEventSeed,
  projectCell,
  stringToHash,
  type ActionHash,
} from '#lib/holo/index.ts';

import { communityEvents, sendEventInvite } from './events.ts';
import { myAgent, namesOf } from './people.ts';
import { getProject } from './projects.ts';

/**
 * Closed communities (ADR 0068 decisions 1 and 5): each is a clone with a private seed, as
 * private events are. Nothing lists them, so you find yours from the clones on your node;
 * others learn the seed from an invite link or an inbox invite.
 */

/** Seeds on this node that turned out not to be communities (events), so they're read once. */
const notCommunities = new Set<string>();

/** web's createCommunity with Openness "Private": a new clone founded with a closed charter. */
export async function createClosedCommunity(
  handle: string,
  description: string,
  organized: boolean,
): Promise<Scope> {
  const me = await myAgent();
  const seed = newPrivateEventSeed(me, organized);
  const cell = await projectCell(seed, handle.trim());
  await callCell<ActionHash>(cell, 'project', 'create_charter', {
    title: handle.trim(),
    description: description.trim(),
    mode: 'community',
    subtype: 'closed',
    location_label: '',
    phase: '',
    tags: [],
  });
  // Your other devices learn it from your inbox, as private events do (ADR 0061).
  await sendEventInvite(me, seed).catch(() => {});
  return {
    id: seed,
    kind: 'community',
    handle: handle.trim(),
    slug: slugOf(handle),
    description: description.trim(),
    members: 1,
    creator: me,
    createdAt: Date.now() * 1000,
    networkSeed: seed,
    isPrivate: true,
  };
}

/** A closed community by its seed (an invite link, or one of your clones). */
export async function closedCommunity(seed: string): Promise<Scope | null> {
  return (await read(seed))?.scope ?? null;
}

async function read(seed: string): Promise<{ scope: Scope; amMember: boolean } | null> {
  if (!isPrivateSeed(seed) || notCommunities.has(seed)) return null;
  const p = await getProject(seed);
  if (p.charter?.mode !== 'community') {
    if (p.charter) notCommunities.add(seed);
    return null;
  }
  const scope: Scope = {
    id: seed,
    kind: 'community',
    handle: p.charter.title,
    slug: slugOf(p.charter.title),
    description: p.charter.description,
    members: p.members.length,
    creator: p.creator,
    createdAt: p.createdAt,
    networkSeed: seed,
    isPrivate: true,
  };
  return { scope, amMember: p.amMember };
}

/** The closed communities you're in, from the clones on your node. */
export async function myClosedCommunities(): Promise<Scope[]> {
  const seeds = (await clonedSeeds('project')).filter(
    (s) => isPrivateSeed(s) && !notCommunities.has(s),
  );
  const found = await Promise.allSettled(
    seeds.map(async (seed) => {
      const found = await read(seed);
      return found?.amMember ? found.scope : null;
    }),
  );
  return found.flatMap((r) => (r.status === 'fulfilled' && r.value ? [r.value] : []));
}

/** web's inviteUserToCommunity: the seed goes to their inbox; they join once they accept. */
export const inviteToCommunity = (agent: string, seed: string): Promise<void> =>
  sendEventInvite(agent, seed);

/** web's "Username to make organizer" (organizer-run communities, ADR 0061's grant). */
export async function addCommunityOrganizer(seed: string, agent: string): Promise<void> {
  const cell = await projectCell(seed, 'Community');
  await callCell<ActionHash>(cell, 'project', 'add_organizer', stringToHash(agent));
}

/** A private community's feed (ADR 0069): the events listed on it, read from their own
 * networks (the index never sees them). Ones that haven't reached this node yet are skipped. */
export async function communityEventCards(community: string): Promise<ProjectCard[]> {
  const seeds = await communityEvents(community);
  const found = await Promise.allSettled(seeds.map((seed) => getProject(seed)));
  const events = found.flatMap((r) =>
    r.status === 'fulfilled' && r.value.charter ? [r.value] : [],
  );
  const names = await namesOf([...new Set(events.map((p) => p.creator))]).catch(
    () => ({}) as Record<string, string>,
  );
  return events.map((p) => eventCard(p, names[p.creator] ?? ''));
}

function eventCard(p: Project, creatorName: string): ProjectCard {
  const c = p.charter as NonNullable<Project['charter']>;
  return {
    id: p.id,
    networkSeed: p.networkSeed,
    creator: p.creator,
    creatorName,
    title: c.title,
    summary: summaryOf(c.description),
    mode: c.mode,
    subtype: projectSubtype(c.subtype),
    locationLabel: c.location_label,
    phase: p.stage,
    tags: c.tags,
    members: p.members.length,
    demand: p.demand,
    opposition: p.opposition,
    createdAt: p.createdAt,
    lastActivityAt: p.lastActivityAt ?? p.createdAt,
    latestUpdate: p.updates[0]?.body ?? null, // newest first
    commentCount: 0,
  };
}
