import type { ProposalKind } from '#lib/domain/governance.ts';
import type { Person } from '#lib/domain/people.ts';
import type { Project } from '#lib/domain/projects.ts';
import { activityAddedNotice, phaseVoteNotice } from '#lib/services/pushNotices.ts';

/** A phase move asks the other members to vote: tell them (ADR 0108). Best effort. */
export function tellMembers(
  d: {
    id: string;
    project?: Project | undefined;
    members: Person[];
    phaseLabel: (id: string) => string;
  },
  kind: ProposalKind,
): void {
  if (!('Phase' in kind) || !d.project) return;
  const title = d.project.charter?.title ?? 'A project';
  const to = d.members.map((m) => m.agent);
  void phaseVoteNotice(to, d.id, title, d.phaseLabel(kind.Phase.to));
}

/** A new activity tells the other members (ADR 0108). Best effort. */
export const tellActivity =
  (d: { id: string; project?: Project | undefined; members: Person[] }) =>
  (a: { id: string; title: string }): void => {
    const title = d.project?.charter?.title ?? 'A project';
    void activityAddedNotice(
      d.members.map((m) => m.agent),
      d.id,
      title,
      a,
    );
  };
