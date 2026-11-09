import {
  railItems,
  type RailItem,
  type RailLink,
  type RailProject,
} from '#lib/domain/activityRail.ts';
import { railHistory } from '#lib/domain/activityRailHistory.ts';
import { requestRailItems } from '#lib/domain/requestRail.ts';
import type { VoteChoice } from '#lib/domain/governance.ts';
import type { SideStatus } from '#lib/domain/links.ts';
import type { CharterMode } from '#lib/domain/projects.ts';
import { isSoftwareProject } from '#lib/domain/software.ts';
import {
  callCell,
  hashToString,
  projectCell,
  type ActionHash,
  type CellId,
} from '#lib/holo/index.ts';

import { listActivities } from './activities.ts';
import { listProposals } from './governance.ts';
import { getModeration } from './moderation.ts';
import { listPlans } from './plans.ts';
import { namesOf, myAgent } from './people.ts';
import { listingTitle, myProjectSeeds } from './projects.ts';
import { myServiceRequests } from './serviceRequests.ts';
import { loadSoftware } from './software.ts';

/** The part of `get_project` the rail reads. */
interface GlanceWire {
  charter: { title: string; mode: CharterMode; subtype: string } | null;
  am_member: boolean;
  phase: string;
  phases: { id: string; label: string }[];
}

/** The part of `link_sides` the rail reads (this project's halves of link requests). */
interface SidesWire {
  halves: {
    id: ActionHash;
    created_at: number;
    request: { other: { listing: ActionHash }; sever: ActionHash | null };
    tally: { yes: number; no: number; my_vote: VoteChoice | null; settled: boolean };
    status: SideStatus;
  }[];
}

/** Listing titles don't change; the rail refreshes every minute, so keep the ones found. */
const titles = new Map<string, string>();

async function titleOf(listing: string): Promise<string> {
  const known = titles.get(listing);
  if (known) return known;
  const found = await listingTitle(listing).catch(() => null);
  if (found) titles.set(listing, found);
  return found ?? 'linked record';
}

async function railLinks(cell: CellId): Promise<RailLink[]> {
  const sides = await callCell<SidesWire>(cell, 'governance', 'link_sides');
  return Promise.all(
    sides.halves.map(async (h) => ({
      id: hashToString(h.id),
      sever: h.request.sever !== null,
      otherTitle: await titleOf(hashToString(h.request.other.listing)),
      createdAt: h.created_at,
      yes: h.tally.yes,
      no: h.tally.no,
      status: h.status,
      settled: h.tally.settled,
      myVote: h.tally.my_vote,
    })),
  );
}

/**
 * web's bootstrap `activityRail` and `activityRailHistory`, built from the projects this
 * person is in, as notifications are (ADR 0046, 0065).
 */
export async function myActivityRail(): Promise<{ items: RailItem[]; history: RailItem[] }> {
  const seeds = await myProjectSeeds();
  const [settled, requests] = await Promise.all([
    Promise.allSettled(seeds.map(railProject)),
    railRequests().catch(() => []),
  ]);
  const projects = settled.flatMap((r) => (r.status === 'fulfilled' && r.value ? [r.value] : []));
  const now = Date.now() * 1000;
  return {
    items: [...railItems(projects, now), ...requests],
    history: railHistory(projects, now),
  };
}

/** Open requests to the services you run (ADR 0076), from your direct chats. */
async function railRequests(): Promise<RailItem[]> {
  const [me, requests] = await Promise.all([myAgent(), myServiceRequests()]);
  const asking = requests.filter((r) => r.status === 'open' && r.requester !== me);
  if (asking.length === 0) return [];
  const names = await namesOf([...new Set(asking.map((r) => r.requester))]);
  return requestRailItems(asking, me, names);
}

async function railProject(seed: string): Promise<RailProject | null> {
  const cell = await projectCell(seed, 'Project');
  const [project, moderation] = await Promise.all([
    callCell<GlanceWire | null>(cell, 'project', 'get_project'),
    getModeration({ seed }).catch(() => null),
  ]);
  if (!project?.charter || !project.am_member) return null;
  // web ends a removed project's memberships, open votes and sign-ups (ADR 0071).
  if (moderation?.state === 'removed') return null;
  const [activities, proposals, board, links, software] = await Promise.all([
    listActivities({ seed }).catch(() => []),
    listProposals({ seed }).catch(() => []),
    listPlans({ seed }).catch(() => null),
    railLinks(cell).catch(() => []),
    isSoftwareProject(project.charter) ? railSoftware(seed).catch(() => null) : null,
  ]);
  const labels = new Map(project.phases.map((p) => [p.id, p.label]));
  return {
    seed,
    title: project.charter.title,
    mode: project.charter.mode,
    amMember: project.am_member,
    phase: project.phase,
    phaseLabel: (id) => labels.get(id) ?? id,
    activities,
    proposals,
    plans: board?.plans ?? [],
    links,
    software,
  };
}

/** A software project's pull requests (ADR 0086), with the names merge capability votes cite. */
async function railSoftware(seed: string): Promise<RailProject['software']> {
  const governance = await loadSoftware({ seed });
  const agents = [...new Set(governance.capabilityRequests.map((c) => c.agent))];
  return { governance, names: agents.length ? await namesOf(agents) : {} };
}
