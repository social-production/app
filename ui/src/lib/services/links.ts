import type { VoteChoice } from '#lib/domain/governance.ts';
import type {
  LinkHalf,
  LinkRequest,
  LinksFrame,
  ProjectLink,
  SideStatus,
} from '#lib/domain/links.ts';
import type { Project, ProjectCard } from '#lib/domain/projects.ts';
import {
  callCell,
  hashToString,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { castVote, closeProposal } from './governance.ts';
import { cellOf, feed, getProject } from './projects.ts';

interface RequestWire {
  other: { listing: ActionHash; network_seed: string };
  relationship: string;
  summary: string;
  sever: ActionHash | null;
  source: ActionHash | null;
}

interface HalfWire {
  id: ActionHash;
  author: AgentPubKey;
  created_at: number;
  request: RequestWire;
  tally: {
    yes: number;
    no: number;
    population: number;
    required: number;
    my_vote: VoteChoice | null;
    settled: boolean;
  };
  status: SideStatus;
}

interface SidesWire {
  network_seed: string;
  am_member: boolean;
  halves: HalfWire[];
}

interface PairWire {
  mine: HalfWire;
  theirs: HalfWire | null;
  status: SideStatus;
}

interface ViewWire {
  active: { create: PairWire; sever: PairWire | null }[];
  historical: { create: PairWire; sever: PairWire | null }[];
  pending: PairWire[];
  past: PairWire[];
}

interface Other {
  project: Project | null;
  amMember: boolean;
}

const governance = async <T>(projectId: string, fn: string, payload?: unknown): Promise<T> => {
  const { cell } = await cellOf(projectId);
  return callCell<T>(cell, 'governance', fn, payload);
};

function half(w: HalfWire): LinkHalf {
  return {
    id: hashToString(w.id),
    author: hashToString(w.author),
    createdAt: w.created_at,
    relationship: w.request.relationship,
    summary: w.request.summary,
    yes: w.tally.yes,
    no: w.tally.no,
    population: w.tally.population,
    required: w.tally.required,
    status: w.status,
    myVote: w.tally.my_vote,
  };
}

const votable = (h: LinkHalf | null, member: boolean) =>
  member && h?.status === 'Open' && !h.myVote;

function card(p: PairWire, here: Project, others: Map<string, Other>): LinkRequest {
  const otherId = hashToString(p.mine.request.other.listing);
  const other = others.get(otherId) ?? { project: null, amMember: false };
  const o = other.project;
  const [mine, theirs] = [half(p.mine), p.theirs ? half(p.theirs) : null];
  const proposedHere = p.mine.request.source === null;
  return {
    id: mine.id,
    sever: p.mine.request.sever !== null,
    relationship: mine.relationship,
    summary: mine.summary,
    proposedBy: proposedHere ? mine.author : (theirs?.author ?? mine.author),
    createdAt: mine.createdAt,
    status: p.status,
    proposedHere,
    target: {
      id: otherId,
      title: o?.charter?.title ?? 'Another project',
      description: o?.charter?.description ?? '',
      mode: o?.charter?.mode ?? 'productive',
      stageLabel: o?.stage ?? '',
      locationLabel: o?.charter?.location_label ?? '',
      memberCount: o ? o.members.length : null,
    },
    thisSide: {
      projectId: here.id,
      projectTitle: here.charter?.title ?? '',
      half: mine,
      canVote: votable(mine, here.amMember),
    },
    otherSide: {
      projectId: otherId,
      projectTitle: o?.charter?.title ?? 'Another project',
      half: theirs,
      canVote: votable(theirs, other.amMember),
    },
  };
}

/** The other project's side; unreachable projects show as not received yet. */
async function readOther(id: string): Promise<{ sides: SidesWire | null; other: Other }> {
  try {
    const [project, sides] = await Promise.all([
      getProject(id),
      governance<SidesWire>(id, 'link_sides'),
    ]);
    return { sides, other: { project, amMember: sides.am_member } };
  } catch {
    return { sides: null, other: { project: null, amMember: false } };
  }
}

/** web's links frame: this project's halves paired with each linked project's. */
/** Halves its votes decide but nobody closed yet: a member closes them on load (ADR 0033). */
async function closeSettled(projectId: string, sides: SidesWire | null): Promise<boolean> {
  const settled = sides?.am_member ? sides.halves.filter((h) => h.tally.settled) : [];
  await Promise.all(settled.map((h) => closeProposal(projectId, hashToString(h.id))));
  return settled.length > 0;
}

export async function listLinks(here: Project, closing = true): Promise<LinksFrame> {
  const mine = await governance<SidesWire>(here.id, 'link_sides');
  const ids = [...new Set(mine.halves.map((h) => hashToString(h.request.other.listing)))];
  const read = await Promise.all(ids.map(async (id) => ({ id, ...(await readOther(id)) })));
  if (closing) {
    const closed = await Promise.all([
      closeSettled(here.id, mine),
      ...read.map((r) => closeSettled(r.id, r.sides)),
    ]);
    if (closed.some(Boolean)) return listLinks(here, false);
  }
  const others = new Map(read.map((r) => [r.id, r.other]));
  const sides = read.flatMap((r) => (r.sides ? [r.sides] : []));
  const view = await governance<ViewWire>(here.id, 'list_links', sides);
  const link = (l: ViewWire['active'][number]): ProjectLink => ({
    id: hashToString(l.create.mine.id),
    create: card(l.create, here, others),
    sever: l.sever ? card(l.sever, here, others) : null,
  });
  return {
    active: view.active.map(link),
    historical: view.historical.map(link),
    pending: view.pending.map((p) => card(p, here, others)).reverse(),
    past: view.past.map((p) => card(p, here, others)),
    canPropose: here.amMember,
  };
}

/** File this project's half, then mirror it into the other project (ADR 0042). */
async function request(
  here: Project,
  otherId: string,
  summary: string,
  severs: [string, string] | null,
): Promise<void> {
  const other = await cellOf(otherId);
  const link = (
    listing: string,
    seed: string,
    sever: string | null,
    source: ActionHash | null,
  ) => ({
    Link: {
      other: { listing: stringToHash(listing), network_seed: seed },
      relationship: '',
      summary: summary.trim(),
      sever: sever ? stringToHash(sever) : null,
      source,
    },
  });
  const id = await governance<ActionHash>(
    here.id,
    'propose',
    link(otherId, other.seed, severs?.[0] ?? null, null),
  );
  await callCell<ActionHash>(
    other.cell,
    'governance',
    'propose',
    link(here.id, here.networkSeed, severs?.[1] ?? null, id),
  );
}

/** web's "Create link vote". */
export function proposeLink(here: Project, otherId: string, summary: string): Promise<void> {
  return request(here, otherId, summary, null);
}

/** web's "Start sever vote" (both projects' create halves are cited). */
export async function proposeSever(here: Project, link: ProjectLink): Promise<void> {
  const theirs = link.create.otherSide.half;
  if (!theirs) throw new Error("The other project's side of this link hasn't arrived yet.");
  await request(here, link.create.target.id, '', [link.id, theirs.id]);
}

/** A vote on one side's half, in that side's project. Votes are final (ADR 0033). */
export async function voteLink(projectId: string, half: string, choice: VoteChoice): Promise<void> {
  await castVote(projectId, half, choice);
}

/** web's link composer search: projects by name, not this one. */
export async function searchProjects(q: string, exclude: string): Promise<ProjectCard[]> {
  const { items } = await feed({ sort: 'recent', window: 'all', q, offset: 0 });
  return items.filter((p) => p.id !== exclude).slice(0, 8);
}
