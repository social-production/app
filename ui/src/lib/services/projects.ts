import {
  summaryOf,
  type Charter,
  type FeedSort,
  type FeedWindow,
  type Project,
  type ProjectCard,
  type Stance,
} from '#lib/domain/projects.ts';
import {
  callCell,
  callZome,
  hashToString,
  newProjectSeed,
  projectCell,
  projectFeed,
  stringToHash,
  type ActionHash,
  type AgentPubKey,
} from '#lib/holo/index.ts';

import { myAgent } from './people.ts';

interface ListingWire {
  id: ActionHash;
  creator: AgentPubKey;
  listing: { network_seed: string; title: string };
}

interface ProjectWire {
  creator: AgentPubKey;
  charter: Charter | null;
  members: AgentPubKey[];
  demand: number;
  opposition: number;
  am_member: boolean;
  my_signal: Stance | null;
  created_at: number | null;
  last_activity_at: number | null;
}

const directory = <T>(fn: string, payload?: unknown) =>
  callZome<T>('directory', 'identity', fn, payload);

export interface FeedQuery {
  sort: FeedSort;
  window: FeedWindow;
  q: string;
  offset: number;
}

export function feed(query: FeedQuery): Promise<{ items: ProjectCard[]; total: number }> {
  const tzOffset = String(-new Date().getTimezoneOffset());
  return projectFeed({ ...query, offset: String(query.offset), tzOffset });
}

/** How many more projects you may start this week; `null` means no limit (Full members). */
export async function projectAllowance(): Promise<number | null> {
  return (await directory<{ remaining: number | null }>('project_allowance')).remaining;
}

/** Start a project: its own network (clone), its charter, then its directory listing. */
export async function createProject(charter: Charter): Promise<string> {
  if ((await projectAllowance()) === 0) {
    throw new Error('New members can start 2 projects per 7 days; full members have no limit.');
  }
  const seed = newProjectSeed(await myAgent());
  const cell = await projectCell(seed, charter.title);
  await callCell<ActionHash>(cell, 'project', 'create_charter', charter);
  const id = await directory<ActionHash>('create_listing', listingOf(seed, charter));
  return hashToString(id);
}

async function listing(id: string): Promise<ListingWire> {
  const found = await directory<ListingWire | null>('get_listing', stringToHash(id));
  if (!found) throw new Error("That project isn't listed (yet). Try again in a moment.");
  return found;
}

async function cellOf(id: string) {
  const l = await listing(id);
  return {
    cell: await projectCell(l.listing.network_seed, l.listing.title),
    seed: l.listing.network_seed,
  };
}

export async function getProject(id: string): Promise<Project> {
  const { cell, seed } = await cellOf(id);
  const p = await callCell<ProjectWire | null>(cell, 'project', 'get_project');
  if (!p) throw new Error('That project has no charter yet.');
  return {
    id,
    networkSeed: seed,
    creator: hashToString(p.creator),
    charter: p.charter,
    members: p.members.map(hashToString),
    demand: p.demand,
    opposition: p.opposition,
    amMember: p.am_member,
    mySignal: p.my_signal,
    createdAt: p.created_at ?? 0,
    lastActivityAt: p.last_activity_at,
  };
}

export async function joinProject(id: string, join: boolean): Promise<void> {
  const { cell } = await cellOf(id);
  await callCell<null>(cell, 'project', join ? 'join' : 'leave');
}

export async function setSignal(id: string, stance: Stance | null): Promise<void> {
  const { cell } = await cellOf(id);
  await callCell<null>(cell, 'project', 'set_signal', stance);
}

/** Creator only (until M3 proposals): the charter, and the listing kept in step. */
export async function updateCharter(id: string, charter: Charter): Promise<void> {
  const { cell, seed } = await cellOf(id);
  await callCell<ActionHash>(cell, 'project', 'update_charter', charter);
  await directory<ActionHash>('update_listing', {
    id: stringToHash(id),
    listing: listingOf(seed, charter),
  });
}

function listingOf(seed: string, c: Charter) {
  return {
    network_seed: seed,
    title: c.title,
    summary: summaryOf(c.description),
    mode: c.mode,
    subtype: c.subtype,
    location_label: c.location_label,
    phase: c.phase,
    tags: c.tags,
  };
}
