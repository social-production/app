/** Mirrors the project DNA's zome and the gateway's project index. */

import type { ProjectUpdate } from './governance.ts';

export type ProjectMode = 'productive' | 'collective-service' | 'personal-service';
export type ProjectSubtype = 'standard' | 'software' | 'asset-management';
export type FeedSort = 'trending' | 'recent' | 'top' | 'oldest';
export type FeedWindow = 'today' | 'week' | 'month' | 'all';
export type Stance = 'Demand' | 'Oppose';

export interface Charter {
  title: string;
  description: string;
  mode: ProjectMode;
  subtype: ProjectSubtype;
  location_label: string;
  phase: string;
  tags: string[];
}

/** One feed card (gateway `ProjectCard`). */
export interface ProjectCard {
  id: string;
  networkSeed: string;
  creator: string;
  /** The creator's username, from the node's index (empty if they have no profile yet). */
  creatorName: string;
  title: string;
  summary: string;
  mode: ProjectMode;
  subtype: ProjectSubtype;
  locationLabel: string;
  phase: string;
  tags: string[];
  members: number;
  demand: number;
  opposition: number;
  createdAt: number;
  lastActivityAt: number;
}

export interface Project {
  id: string;
  networkSeed: string;
  creator: string;
  charter: Charter | null;
  members: string[];
  demand: number;
  opposition: number;
  amMember: boolean;
  mySignal: Stance | null;
  /** Holochain micros. */
  createdAt: number;
  lastActivityAt: number | null;
  /** web's phase id ("phase-1"…) and its label for this mode ("Production Plan"). */
  phase: string;
  stage: string;
  /** The phases this project shows, and the one a proposal may move it to next. */
  phases: PhaseStep[];
  nextPhase: PhaseStep | null;
  updates: ProjectUpdate[];
}

export interface PhaseStep {
  id: string;
  label: string;
}

// Limits mirror sp_core::project (validation enforces them; the UI just helps).
export const TITLE_MAX = 200;
export const DESCRIPTION_MAX = 20_000;
export const SUMMARY_MAX = 500;
export const LOCATION_LABEL_MAX = 160;
export const TAGS_MAX = 10;
export const TAG_MAX = 40;

export const MODE_LABELS: Record<ProjectMode, string> = {
  productive: 'Productive',
  'collective-service': 'Collective service',
  'personal-service': 'Personal service',
};

/** The listing summary: the description's first paragraph, trimmed to fit. */
export function summaryOf(description: string): string {
  const first = description.trim().split(/\n\s*\n/)[0] ?? '';
  return first.length > SUMMARY_MAX ? `${first.slice(0, SUMMARY_MAX - 1)}…` : first;
}

/** The left accent bar colour per mode (web's surfaceTypeAccent). */
export function modeAccent(mode: ProjectMode): string {
  if (mode === 'productive') return 'var(--type-accent-project-production)';
  return mode === 'personal-service'
    ? 'var(--type-accent-project-personal-service)'
    : 'var(--type-accent-project-service)';
}

/** Personal-service projects take no demand/opposition signals (as on web). */
export function takesSignals(mode: ProjectMode): boolean {
  return mode !== 'personal-service';
}

/** The fields the edit sheet changes (web's ProposeEditSheet). */
export interface CharterDraft {
  title: string;
  description: string;
  location: string;
}

export interface ProjectTypeOption {
  value: ProjectMode;
  label: string;
  summary: string;
}

/** The create flow's type choices, worded as on web (features/projects/projectMode.ts). */
export const PROJECT_TYPES: ProjectTypeOption[] = [
  {
    value: 'productive',
    label: 'Productive project',
    summary:
      'Build or produce something new — e.g. construction, materials, tools, or software from scratch.',
  },
  {
    value: 'collective-service',
    label: 'Collective service',
    summary:
      'Maintain, update, or operate something already in use — e.g. keep software running, dinner service, shared care.',
  },
  {
    value: 'personal-service',
    label: 'Personal service',
    summary: 'One person offering time or skill through calendar slots and/or written requests.',
  },
];

/** Where a new project happens (web's location intent). */
export type LocationChoice = 'physical' | 'online' | 'later';

/** A detail page's tabs (web's detailTabs.ts); each appears once its milestone lands. */
export type DetailTabId = 'context' | 'participation' | 'chat' | 'links' | 'history';
