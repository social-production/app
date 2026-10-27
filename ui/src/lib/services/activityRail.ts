import {
  railItems,
  type RailItem,
  type RailLink,
  type RailProject,
} from '#lib/domain/activityRail.ts';
import { railHistory } from '#lib/domain/activityRailHistory.ts';
import type { VoteChoice } from '#lib/domain/governance.ts';
import type { SideStatus } from '#lib/domain/links.ts';
import type { CharterMode } from '#lib/domain/projects.ts';
import {
  callCell,
  clonedSeeds,
  hashToString,
  projectCell,
  type ActionHash,
  type CellId,
} from '#lib/holo/index.ts';

import { listActivities } from './activities.ts';
import { listProposals } from './governance.ts';
import { listPlans } from './plans.ts';
import { listingTitle } from './projects.ts';

/** The part of `get_project` the rail reads. */
interface GlanceWire {
  charter: { title: string; mode: CharterMode } | null;
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
 * web's bootstrap `activityRail` and `activityRailHistory`, built from the projects open on
 * this node, as notifications are (ADR 0046): reading any other project would join its network.
 */
export async function myActivityRail(): Promise<{ items: RailItem[]; history: RailItem[] }> {
  const seeds = await clonedSeeds('project');
  const settled = await Promise.allSettled(seeds.map(railProject));
  const projects = settled.flatMap((r) => (r.status === 'fulfilled' && r.value ? [r.value] : []));
  const now = Date.now() * 1000;
  return { items: railItems(projects, now), history: railHistory(projects, now) };
}

async function railProject(seed: string): Promise<RailProject | null> {
  const cell = await projectCell(seed, 'Project');
  const project = await callCell<GlanceWire | null>(cell, 'project', 'get_project');
  if (!project?.charter || !project.am_member) return null;
  const [activities, proposals, board, links] = await Promise.all([
    listActivities({ seed }).catch(() => []),
    listProposals({ seed }).catch(() => []),
    listPlans({ seed }).catch(() => null),
    railLinks(cell).catch(() => []),
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
  };
}
