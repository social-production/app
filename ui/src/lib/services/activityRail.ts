import { railItems, type RailItem, type RailProject } from '#lib/domain/activityRail.ts';
import { railHistory } from '#lib/domain/activityRailHistory.ts';
import type { ProjectMode } from '#lib/domain/projects.ts';
import { callCell, clonedSeeds, projectCell } from '#lib/holo/index.ts';

import { listActivities } from './activities.ts';
import { listProposals } from './governance.ts';
import { listPlans } from './plans.ts';

/** The part of `get_project` the rail reads. */
interface GlanceWire {
  charter: { title: string; mode: ProjectMode } | null;
  am_member: boolean;
  phase: string;
  phases: { id: string; label: string }[];
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
  const [activities, proposals, board] = await Promise.all([
    listActivities({ seed }).catch(() => []),
    listProposals({ seed }).catch(() => []),
    listPlans({ seed }).catch(() => null),
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
  };
}
