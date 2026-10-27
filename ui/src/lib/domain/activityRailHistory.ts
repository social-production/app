/**
 * The right rail's History (web-backend's `bootstrap/activity_rail_history.py`): activities
 * that ended, roles you hold now, and the votes you cast, newest first.
 */
import {
  activityQuery,
  inActivityPhase,
  linkKind,
  linkQuery,
  voteQuery,
  type RailItem,
  type RailProject,
  type RailVoteKind,
} from './activityRail.ts';
import type { Proposal, ProposalStatus, VoteChoice } from './governance.ts';

/** web-backend's limits. */
const PAST_LIMIT = 20;
const SIGNUP_LIMIT = 10;
const VOTE_LIMIT = 12;
const HISTORY_LIMIT = 40;
const UPDATE_TITLE_MAX = 80;

export type HistoryFilter = 'all' | 'votes' | 'activities' | 'help' | 'requests';

export function railHistory(projects: RailProject[], nowMicros: number): RailItem[] {
  const members = projects.filter((p) => p.amMember);
  const items = [...pastActivities(members, nowMicros), ...castVotes(projects)];
  const seen = new Set(items.map((i) => i.id));
  items.push(...currentSignups(members, nowMicros).filter((i) => !seen.has(i.id)));
  return items.sort((a, b) => b.createdAt - a.createdAt).slice(0, HISTORY_LIMIT);
}

function pastActivities(projects: RailProject[], now: number): RailItem[] {
  return projects
    .flatMap((p) => p.activities.filter((a) => a.endsAt <= now).map((a) => ({ p, a })))
    .sort((x, y) => y.a.endsAt - x.a.endsAt)
    .slice(0, PAST_LIMIT)
    .map(({ p, a }) => ({
      id: a.id,
      seed: p.seed,
      kind: 'project' as const,
      title: a.title,
      query: activityQuery(a.id),
      meta: p.title,
      createdAt: a.startsAt,
      projectMode: p.mode,
      activityId: a.id,
      viewerParticipated: a.myRole !== null,
    }));
}

/** web's `_append_current_role_signups`: upcoming activities you hold a role in. */
function currentSignups(projects: RailProject[], now: number): RailItem[] {
  return projects
    .filter(inActivityPhase)
    .flatMap((p) =>
      p.activities.filter((a) => a.endsAt > now && a.myRole !== null).map((a) => ({ p, a })),
    )
    .sort((x, y) => y.a.startsAt - x.a.startsAt)
    .slice(0, SIGNUP_LIMIT)
    .map(({ p, a }) => ({
      id: `signup-project-activity-${a.id}`,
      seed: p.seed,
      kind: 'project' as const,
      title: a.title,
      query: activityQuery(a.id),
      meta: `You signed up · ${p.title}`,
      createdAt: a.startsAt,
      projectMode: p.mode,
      activityId: a.id,
      viewerParticipated: true,
    }));
}

const VOTE_LABELS: Record<VoteChoice, string> = { Yes: 'Yes', No: 'No', Neutral: 'Neutral' };

function outcomeOf(status: ProposalStatus, leading = false): 'passed' | 'failed' | 'open' {
  if (leading || status === 'Passed') return 'passed';
  return status === 'Rejected' ? 'failed' : 'open';
}

interface Cast {
  id: string;
  p: RailProject;
  kind: RailVoteKind;
  query: string;
  title: string;
  meta: string;
  createdAt: number;
  vote: VoteChoice;
  outcome: 'passed' | 'failed' | 'open';
}

const truncate = (text: string, max: number): string =>
  text.length > max ? `${text.slice(0, max - 1).trimEnd()}…` : text;

/** A cast proposal vote as web words its history row (title, meta) per kind. */
function proposalCast(p: RailProject, v: Proposal & { myVote: VoteChoice }): Cast | null {
  const base = {
    p,
    query: '',
    createdAt: v.createdAt,
    vote: v.myVote,
    outcome: outcomeOf(v.status),
  };
  const cast = (kind: RailVoteKind, id: string, title: string, meta: string): Cast => ({
    ...base,
    id: `vote-project-${id}-${v.id}`,
    kind,
    query: voteQuery(kind, v.id),
    title,
    meta,
  });
  if ('Phase' in v.kind) {
    return cast('phase_change', 'phase', p.title, `Advance to ${p.phaseLabel(v.kind.Phase.to)}`);
  }
  if ('Update' in v.kind) {
    return cast('update', 'update', truncate(v.kind.Update.body, UPDATE_TITLE_MAX), p.title);
  }
  if ('Edit' in v.kind) return cast('edit', 'edit', v.kind.Edit.title ?? p.title, p.title);
  return null;
}

function castVotes(projects: RailProject[]): RailItem[] {
  const proposals = projects.flatMap((p) =>
    p.proposals.flatMap((v) =>
      v.myVote === null ? [] : [proposalCast(p, { ...v, myVote: v.myVote })],
    ),
  );
  const plans = projects.flatMap((p) =>
    p.plans.flatMap((x): Cast[] =>
      x.tally.myVote === null
        ? []
        : [
            {
              id: `vote-project-plan-${x.id}`,
              p,
              kind: 'plan',
              query: voteQuery('plan', x.id),
              title: x.plan.title,
              meta: p.title,
              createdAt: x.createdAt,
              vote: x.tally.myVote,
              outcome: outcomeOf(x.tally.status, x.leader === 'Leading'),
            },
          ],
    ),
  );
  // web-backend keeps 12 of each kind; link and sever votes share one list.
  const byKind = new Map<string, Cast[]>();
  for (const c of [...plans, ...proposals, ...linkCasts(projects)]) {
    const key = c?.kind === 'link_sever' ? 'link' : c?.kind;
    if (c && key) byKind.set(key, [...(byKind.get(key) ?? []), c]);
  }
  return [...byKind.values()].flatMap((rows) =>
    rows
      .sort((a, b) => b.createdAt - a.createdAt)
      .slice(0, VOTE_LIMIT)
      .map((c) => ({
        id: c.id,
        seed: c.p.seed,
        kind: 'vote' as const,
        title: c.title,
        query: c.query,
        meta: `You voted ${VOTE_LABELS[c.vote]} · ${c.meta}`,
        createdAt: c.createdAt,
        projectMode: c.p.mode,
        voteKindLabel: c.kind,
        viewerParticipated: true,
        outcome: c.outcome,
      })),
  );
}

/**
 * Link votes you cast, as web words them ("You voted Yes · Link vote"). The outcome is this
 * project's half's; web reads the whole request's (both halves).
 */
function linkCasts(projects: RailProject[]): Cast[] {
  return projects.flatMap((p) =>
    p.links.flatMap((l): Cast[] =>
      l.myVote === null
        ? []
        : [
            {
              id: `vote-${linkKind(l)}-${l.id}`,
              p,
              kind: linkKind(l),
              query: linkQuery(l.id),
              title: p.title,
              meta: l.sever ? 'Sever link' : 'Link vote',
              createdAt: l.createdAt,
              vote: l.myVote,
              outcome:
                l.status === 'Approved' ? 'passed' : l.status === 'Rejected' ? 'failed' : 'open',
            },
          ],
    ),
  );
}

/** web's `historyMatchesFilter` (help and service requests don't exist here yet). */
export function historyMatches(item: RailItem, filter: HistoryFilter): boolean {
  if (filter === 'votes') return item.kind === 'vote' && item.viewerParticipated !== false;
  if (filter === 'activities') return item.kind === 'project';
  if (filter === 'help' || filter === 'requests') return false;
  return true;
}

/** web's `historyOutcome`. */
export function historyOutcome(item: RailItem): string {
  if (item.kind === 'vote') {
    if (item.outcome === 'passed') return 'Passed';
    if (item.outcome === 'failed') return 'Did not pass';
    return 'Still open';
  }
  return item.viewerParticipated ? 'You participated' : 'You did not participate';
}
