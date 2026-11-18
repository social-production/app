import type { Person } from '#lib/domain/people.ts';
import type { Proposal } from '#lib/domain/governance.ts';
import type { Moderation } from '#lib/domain/moderation.ts';
import { isProjectMode, type Project } from '#lib/domain/projects.ts';
import type { SoftwareGovernance } from '#lib/domain/software.ts';
import { getBoard } from '#lib/services/board.ts';
import { planLeadChanges } from '#lib/services/governance.ts';
import { myAgent } from '#lib/services/people.ts';
import {
  phaseDoneNotice,
  awaitingMergeNotice,
  planLeadNotice,
  prApprovedNotice,
  removedNotice,
} from '#lib/services/pushNotices.ts';

/**
 * Push for decisions (ADR 0108): whoever's write decided a vote tells the people its card is
 * for, once. A vote is decided by the vote that tips it or, when nobody saw it tip, by the
 * member who loads the page next and writes its Outcome; that write is this device's.
 */
const told = new Set<string>();
const RECENT_MICROS = 10 * 60 * 1_000_000;
const recent = (micros: number): boolean => Date.now() * 1000 - micros < RECENT_MICROS;

function once(key: string): boolean {
  if (told.has(key)) return false;
  told.add(key);
  return true;
}

/** Phase moves and returns among `decided` (ids this device just closed or tipped). */
export function tellPhases(
  d: { id: string; members: Person[]; proposals: Proposal[]; phaseLabel: (id: string) => string },
  decided: string[],
): void {
  for (const p of d.proposals) {
    if (!decided.includes(p.id) || p.status !== 'Passed' || !once(`phase:${p.id}`)) continue;
    const to = 'Phase' in p.kind ? p.kind.Phase.to : 'Return' in p.kind ? p.kind.Return.to : null;
    if (to === null) continue;
    void phaseDoneNotice(
      d.members.map((m) => m.agent),
      d.id,
      d.phaseLabel(to),
    );
  }
}

/** Plans that took the lead by this device's close or vote, just now. */
export async function tellPlanLeads(project: string, event: boolean): Promise<void> {
  const [changes, me] = await Promise.all([planLeadChanges(project), myAgent()]);
  for (const c of changes) {
    if (c.by !== me || !recent(c.at) || !once(`lead:${c.plan}:${c.at}`)) continue;
    void planLeadNotice(c.author, project, event);
  }
}

/** Pull requests approved by this device's close or vote, just now. */
export async function tellApprovals(project: string, g: SoftwareGovernance): Promise<void> {
  const me = await myAgent();
  for (const pr of g.pullRequests) {
    const a = pr.approved;
    if (!a || a.by !== me || !recent(a.at) || !once(`pr:${pr.id}`)) continue;
    void prApprovedNotice(pr.author, project, pr.id);
    // Whoever can merge is asked to (ADR 0086): the merge happens on the repository by hand.
    // On #platform that is the platform moderators (ADR 0095).
    const mergers = g.platform
      ? (await getBoard()).moderators.map((m) => m.agent)
      : g.mergers.map((m) => m.agent).filter((m) => m !== pr.author);
    void awaitingMergeNotice(mergers, project, pr.id, pr.title);
  }
}

/** After this device's write: the phases among `decided`, and plan leads when `plans`. */
export function tellDecisions(
  d: Parameters<typeof tellPhases>[0] & { project?: Project | undefined },
  decided: string[],
  plans: boolean,
): void {
  tellPhases(d, decided);
  if (plans) void tellPlanLeads(d.id, d.project?.charter?.mode === 'event').catch(() => {});
}

/** A report this device's vote decided removed the project: its members hear (projects only,
 * as web's `prj-removed`). */
export function tellRemoved(
  d: { members: Person[]; project?: Project | undefined },
  m: Moderation,
): void {
  const mode = d.project?.charter?.mode;
  if (!m.report || !mode || !isProjectMode(mode) || !once(`removed:${m.report.id}`)) return;
  const title = d.project?.charter?.title ?? 'A project';
  const reason = m.report.reason === 'spam' ? 'spam' : 'serious harm';
  void removedNotice(
    d.members.map((p) => p.agent),
    title,
    reason,
  );
}
