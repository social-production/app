/**
 * The right rail's Help Requests (web-backend's `bootstrap/activity_rail.py` and
 * `activity_rail_history.py`): requests you made, ones you signed up for, and open ones in
 * your channels and communities. A help request is a commons space with one activity (its
 * time and roles, ADR 0067), so yours and your sign-ups come from the projects you are in and
 * open ones from the feed index. Times are Holochain microseconds.
 */
import { committedOf, type Activity } from './activities.ts';
import type { RailItem, RailProject } from './activityRail.ts';
import { HELP_MODE } from './helpRequests.ts';
import type { ProjectCard } from './projects.ts';

/** web-backend's limits: six of each open kind, ten of each past kind. */
const OPEN_LIMIT = 6;
const PAST_LIMIT = 10;
const BODY_MAX = 200;

export type HelpKind = 'help-request-owned' | 'help-request-signup' | 'help-request-open';

export const isHelpKind = (kind: string): kind is HelpKind => kind.startsWith('help-request');

/** web's `_truncate_update_body`. */
const preview = (text: string): string => {
  const trimmed = text.trim();
  return trimmed.length <= BODY_MAX ? trimmed : `${trimmed.slice(0, BODY_MAX).trimEnd()}…`;
};

/** web's `_help_request_role_summaries` label: places are the roles' limits ("any" adds none). */
export function helpCountLabel(signed: number, needed: number): string {
  return needed > 0 ? `${signed} signed up · ${needed} needed` : `${signed} signed up`;
}

const neededOf = (a: Activity): number => a.roles.reduce((n, r) => n + (r.maximum ?? 0), 0);

interface Mine {
  p: RailProject;
  a: Activity;
  owned: boolean;
}

/** Your help requests and the ones you signed up for, with their one activity. */
function mine(projects: RailProject[], me: string): Mine[] {
  return projects.flatMap((p) => {
    const a = p.activities[0];
    if (p.mode !== HELP_MODE || !p.amMember || !a) return [];
    const owned = p.creator === me;
    return owned || a.myRole !== null ? [{ p, a, owned }] : [];
  });
}

function mineItem({ p, a, owned }: Mine, history: boolean): RailItem {
  return {
    id: `help-${p.seed}`,
    seed: p.seed,
    kind: owned ? 'help-request-owned' : 'help-request-signup',
    title: p.title,
    query: '',
    meta: owned ? 'Your request' : 'You signed up',
    createdAt: a.startsAt,
    ...(history
      ? { viewerParticipated: true }
      : { countLabel: helpCountLabel(committedOf(a), neededOf(a)), body: preview(p.description) }),
    projectMode: HELP_MODE,
  };
}

const soonestFirst = (x: Mine, y: Mine): number => x.a.startsAt - y.a.startsAt;

/** The open cards: yours, then your sign-ups, then open requests in your scopes. */
export function helpRailItems(
  projects: RailProject[],
  open: ProjectCard[],
  me: string,
  now: number,
): RailItem[] {
  const current = mine(projects, me).filter((m) => m.a.endsAt > now);
  const owned = current
    .filter((m) => m.owned)
    .sort(soonestFirst)
    .slice(0, OPEN_LIMIT);
  const signed = current
    .filter((m) => !m.owned)
    .sort(soonestFirst)
    .slice(0, OPEN_LIMIT);
  const taken = new Set(current.map((m) => m.p.seed));
  const others = open
    .filter((c) => c.mode === HELP_MODE && !taken.has(c.networkSeed) && c.creator !== me)
    .filter((c) => (c.help?.endsAt ?? Infinity) > now)
    .sort((x, y) => (x.help?.neededAt ?? 0) - (y.help?.neededAt ?? 0))
    .slice(0, OPEN_LIMIT)
    .map((c): RailItem => ({
      id: `open-help-${c.networkSeed}`,
      seed: c.networkSeed,
      projectId: c.id,
      kind: 'help-request-open',
      title: c.title,
      query: '',
      meta: '',
      createdAt: c.help?.neededAt ?? c.createdAt,
      countLabel: helpCountLabel(c.help?.signupCount ?? 0, c.help?.slotsNeeded ?? 0),
      body: preview(c.summary),
      projectMode: HELP_MODE,
    }));
  return [
    ...owned.map((m) => mineItem(m, false)),
    ...signed.map((m) => mineItem(m, false)),
    ...others,
  ];
}

/**
 * History: your ended requests and ended sign-ups (ten each, newest first), and the ones you
 * are signed up for now (web's `_append_current_role_signups`).
 */
export function helpRailHistory(projects: RailProject[], me: string, now: number): RailItem[] {
  const all = mine(projects, me).sort((x, y) => y.a.startsAt - x.a.startsAt);
  const ended = all.filter((m) => m.a.endsAt <= now);
  const owned = ended.filter((m) => m.owned).slice(0, PAST_LIMIT);
  const signed = ended.filter((m) => !m.owned).slice(0, PAST_LIMIT);
  const current = all.filter((m) => !m.owned && m.a.endsAt > now).slice(0, PAST_LIMIT);
  return [...owned, ...signed, ...current].map((m) => mineItem(m, true));
}
