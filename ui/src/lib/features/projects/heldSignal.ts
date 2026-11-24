import type { Project, Stance } from '#lib/domain/projects.ts';

/**
 * Your support or opposition, just set, held on the page until the project's own read shows
 * it: a network read lags a write, and the page re-reads on every tab (owner, 2026-11-25:
 * "it went away when I changed tab").
 */
export interface HeldSignal {
  mine: Stance | null;
  demand: number;
  opposition: number;
}

/** The project with `next` as your stance, its counts moved to match. */
export function withSignal(project: Project, next: Stance | null): [Project, HeldSignal] {
  const before = project.mySignal;
  const count = (s: Stance, n: number): number => n - Number(before === s) + Number(next === s);
  const held = {
    mine: next,
    demand: count('Demand', project.demand),
    opposition: count('Oppose', project.opposition),
  };
  return [{ ...project, mySignal: next, ...stance(held) }, held];
}

/** A fresh read with the held stance laid over it, until the read shows it. */
export function overHeld(project: Project, held: HeldSignal | null): [Project, HeldSignal | null] {
  if (!held || project.mySignal === held.mine) return [project, null];
  return [{ ...project, mySignal: held.mine, ...stance(held) }, held];
}

function stance(held: HeldSignal): Pick<Project, 'demand' | 'opposition'> {
  return { demand: held.demand, opposition: held.opposition };
}
