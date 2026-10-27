/**
 * The right rail's cards (web's `RightRailActivityItem`, built by web-backend's
 * `bootstrap/activity_rail.py`), derived on this device from the projects open on its node,
 * as notifications are (ADR 0046). Times are Holochain microseconds.
 */
import { committedOf, minimumOf, type Activity } from './activities.ts';
import { zoneOf } from './activityTime.ts';
import type { Proposal, VoteChoice } from './governance.ts';
import type { SideStatus } from './links.ts';
import type { PlanView } from './plans.ts';
import type { ProjectMode } from './projects.ts';

/** web's kinds this app has so far; events, help requests and service requests come later. */
export type RailKind = 'project' | 'vote';
export type RailVoteKind = 'phase_change' | 'plan' | 'update' | 'edit' | 'link' | 'link_sever';

export interface RailItem {
  id: string;
  /** The project's network seed; its page is looked up when the card is opened. */
  seed: string;
  kind: RailKind;
  title: string;
  /** The project page's query, as web's `href` carries it. */
  query: string;
  meta: string;
  createdAt: number;
  countLabel?: string;
  projectMode: ProjectMode;
  activityId?: string;
  voteKindLabel?: RailVoteKind;
  viewerParticipated?: boolean;
  outcome?: 'passed' | 'failed' | 'open';
}

/** What one project gives the rail. */
export interface RailProject {
  seed: string;
  title: string;
  mode: ProjectMode;
  amMember: boolean;
  phase: string;
  phaseLabel: (id: string) => string;
  activities: Activity[];
  proposals: Proposal[];
  plans: PlanView[];
  links: RailLink[];
}

/** This project's half of a link request (its `link_sides`), with the other project's title. */
export interface RailLink {
  id: string;
  sever: boolean;
  otherTitle: string;
  createdAt: number;
  yes: number;
  no: number;
  status: SideStatus;
  settled: boolean;
  myVote: VoteChoice | null;
}

/** web-backend's limits: four activities, three votes of each kind. */
const ACTIVITY_LIMIT = 4;
const VOTE_LIMIT = 3;
/**
 * The rail lists activities only while a project is in its Activity phase. web-backend checks
 * `phase-5` alone, which leaves out personal services (their Activity phase is `phase-1`,
 * ADR 0056); the app counts both (PARITY: owner review).
 */
export const inActivityPhase = (p: Pick<RailProject, 'mode' | 'phase'>): boolean =>
  p.phase === (p.mode === 'personal-service' ? 'phase-1' : 'phase-5');

export const voteQuery = (kind: RailVoteKind, target: string, assess = false): string =>
  `?open=vote&voteKind=${kind}&voteTarget=${encodeURIComponent(target)}${assess ? '&assess=1' : ''}`;

export const activityQuery = (id: string): string => `?activity=${encodeURIComponent(id)}`;

/** web's link vote href: the Links tab with the request highlighted. */
export const linkQuery = (id: string): string => `?tab=links&linkRequest=${encodeURIComponent(id)}`;

export const linkKind = (l: RailLink): RailVoteKind => (l.sever ? 'link_sever' : 'link');

/** web's "2 signed up · 1 needed". */
export function signupLabel(a: Activity): string {
  const signed = committedOf(a);
  const needed = Math.max(0, minimumOf(a) - signed);
  return needed > 0 ? `${signed} signed up · ${needed} needed` : `${signed} signed up`;
}

/** The rail's open cards: upcoming activities, then votes you haven't cast. */
export function railItems(projects: RailProject[], nowMicros: number): RailItem[] {
  const members = projects.filter((p) => p.amMember);
  return [...upcomingActivities(members, nowMicros), ...votesToCast(members)];
}

function upcomingActivities(projects: RailProject[], now: number): RailItem[] {
  return projects
    .filter(inActivityPhase)
    .flatMap((p) => p.activities.filter((a) => a.endsAt > now).map((a) => ({ p, a })))
    .sort((x, y) => x.a.startsAt - y.a.startsAt)
    .slice(0, ACTIVITY_LIMIT)
    .map(({ p, a }) => ({
      id: a.id,
      seed: p.seed,
      kind: 'project' as const,
      title: a.title,
      query: activityQuery(a.id),
      meta: a.place.trim() || (a.online ? 'Online' : ''),
      createdAt: a.startsAt,
      countLabel: signupLabel(a),
      projectMode: p.mode,
      activityId: a.id,
    }));
}

const tally = (t: { yes: number; no: number }): string => `${t.yes} yes / ${t.no} no`;

const newestFirst = <T extends { createdAt: number }>(rows: T[]): T[] =>
  rows.slice().sort((a, b) => b.createdAt - a.createdAt);

function votesToCast(projects: RailProject[]): RailItem[] {
  const open = (p: RailProject, kind: 'Phase' | 'Update' | 'Edit') =>
    p.proposals
      .filter((v) => kind in v.kind && v.status === 'Open' && !v.settled && v.myVote === null)
      .map((v) => ({ p, v, createdAt: v.createdAt }));
  const pick = <T extends { createdAt: number }>(rows: T[]): T[] =>
    newestFirst(rows).slice(0, VOTE_LIMIT);
  const phases = pick(projects.flatMap((p) => open(p, 'Phase'))).map(({ p, v }) => {
    const to = 'Phase' in v.kind ? v.kind.Phase.to : '';
    return proposalItem(p, v, 'phase_change', p.title, `Advance to ${p.phaseLabel(to)}?`);
  });
  const plans = pick(
    projects.flatMap((p) =>
      p.plans
        .filter((x) => x.tally.status === 'Open' && !x.tally.settled && x.tally.myVote === null)
        .map((x) => ({ p, x, createdAt: x.createdAt })),
    ),
  ).map(({ p, x }) => planItem(p, x));
  const updates = pick(projects.flatMap((p) => open(p, 'Update'))).map(({ p, v }) =>
    proposalItem(p, v, 'update', p.title, 'Update request'),
  );
  const edits = pick(projects.flatMap((p) => open(p, 'Edit'))).map(({ p, v }) =>
    proposalItem(p, v, 'edit', p.title, 'Edit request'),
  );
  return [...phases, ...plans, ...updates, ...edits, ...linksToCast(projects)];
}

/**
 * web-backend reads the newest nine open link requests (three times its per-kind limit) and
 * keeps the halves the viewer hasn't voted on: "Link vote · <other project>".
 */
function linksToCast(projects: RailProject[]): RailItem[] {
  const rows = projects.flatMap((p) =>
    p.links
      .filter((l) => l.status === 'Open' && !l.settled && l.myVote === null)
      .map((l) => ({ p, l, createdAt: l.createdAt })),
  );
  return newestFirst(rows)
    .slice(0, VOTE_LIMIT * 3)
    .map(({ p, l }) => ({
      id: l.id,
      seed: p.seed,
      kind: 'vote' as const,
      title: p.title,
      query: linkQuery(l.id),
      meta: `${l.sever ? 'Sever link' : 'Link vote'} · ${l.otherTitle}`,
      createdAt: l.createdAt,
      countLabel: tally(l),
      projectMode: p.mode,
      voteKindLabel: linkKind(l),
    }));
}

function proposalItem(
  p: RailProject,
  v: Proposal,
  kind: RailVoteKind,
  title: string,
  meta: string,
): RailItem {
  return {
    id: v.id,
    seed: p.seed,
    kind: 'vote',
    title,
    query: voteQuery(kind, v.id),
    meta,
    createdAt: v.createdAt,
    countLabel: tally(v),
    projectMode: p.mode,
    voteKindLabel: kind,
  };
}

/** web: rate the criteria first ("Assess and approve"), then the overall vote. */
function planItem(p: RailProject, x: PlanView): RailItem {
  const assessed = x.criteria.some((c) => c.mine !== null);
  return {
    id: x.id,
    seed: p.seed,
    kind: 'vote',
    title: x.plan.title || p.title,
    query: voteQuery('plan', x.id, assessed),
    meta: assessed ? `Approve “${x.plan.title}”?` : 'Assess and approve this plan',
    createdAt: x.createdAt,
    countLabel: tally(x.tally),
    projectMode: p.mode,
    voteKindLabel: 'plan',
  };
}

/**
 * web's `countActionableRailItems`: the activity button's badge counts votes and activities
 * you haven't opened or cleared (web-backend never sends `viewerIsParticipating`).
 */
export function countActionable(
  items: RailItem[],
  dismissed: readonly string[],
  seen: readonly string[],
): number {
  return items.filter((i) => !dismissed.includes(i.id) && !seen.includes(i.id)).length;
}

/** web's `itemKicker`. */
export function railKicker(item: RailItem): string {
  if (item.kind === 'vote') {
    return item.voteKindLabel ? `Vote · ${item.voteKindLabel.replace('_', ' ')}` : 'Vote';
  }
  return 'Project activity';
}

/** web's `formatLocalDateTime`: "Tue, Oct 27, 2026, 6:00 PM AEDT" (the browser's zone). */
export function railTime(micros: number): string {
  const date = new Date(micros / 1000);
  const formatted = date.toLocaleString(undefined, {
    weekday: 'short',
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  });
  const zone = zoneOf(date);
  return zone ? `${formatted} ${zone}` : formatted;
}
