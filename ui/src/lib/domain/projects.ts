/** Mirrors the project DNA's zome and the gateway's project index. */

import type { ProjectUpdate } from './governance.ts';

export type ProjectMode = 'productive' | 'collective-service' | 'personal-service';
/** A charter's mode: a project's, "event" (ADR 0057), "help-request" (ADR 0067), or a
 * channel's or community's (ADR 0068). */
export type CharterMode = ProjectMode | 'event' | 'help-request' | 'channel' | 'community';
export type ProjectSubtype = 'standard' | 'software' | 'asset-management';
export type FeedSort = 'trending' | 'recent' | 'top' | 'oldest';
export type FeedWindow = 'today' | 'week' | 'month' | 'all';
export type Stance = 'Demand' | 'Oppose';

export interface Charter {
  title: string;
  description: string;
  mode: CharterMode;
  /** A project's subtype; a channel's or community's join policy (`open`/`closed`). */
  subtype: ProjectSubtype | 'open' | 'closed';
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
  mode: CharterMode;
  subtype: ProjectSubtype;
  locationLabel: string;
  phase: string;
  tags: string[];
  members: number;
  demand: number;
  opposition: number;
  createdAt: number;
  lastActivityAt: number;
  /** The newest applied update's text (web's `latestDescription`), if any. */
  latestUpdate: string | null;
  /** Messages in the project chat (web's `commentCount`; counts up to the chat's page size). */
  commentCount: number;
  /** A help request's time and sign-ups (ADR 0067; times in Holochain microseconds). */
  help?: {
    neededAt: number | null;
    endsAt: number | null;
    signupCount: number;
    slotsNeeded: number;
  };
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
  /** Earlier phases a return may go back to (web's `revertablePhaseIds`). */
  returns: PhaseStep[];
  updates: ProjectUpdate[];
  /** web's event audience and governance (ADR 0061); projects are public and collaborative. */
  audience: EventAudience;
  governance: EventGovernance;
  /** The creator first, then co-organizers. */
  organizers: string[];
  amOrganizer: boolean;
}

export type EventAudience = 'public' | 'invite_only';
export type EventGovernance = 'collaborative' | 'organizer_controlled';

/** A private event's page id is its seed (`<creator>.private|organized.<nonce>`); a listed
 * project's is its listing id, which has no dot. */
export const isPrivateSeed = (id: string): boolean =>
  /^[^.]+\.(private|organized)\.[^.]+$/.test(id);

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

/** A project proper (not an event, help request, channel or community): it has subtypes. */
export function isProjectMode(mode: CharterMode): mode is ProjectMode {
  return mode === 'productive' || mode === 'collective-service' || mode === 'personal-service';
}

/** A charter's subtype as a project subtype (a scope's join policy isn't one). */
export function projectSubtype(subtype: Charter['subtype']): ProjectSubtype {
  return subtype === 'open' || subtype === 'closed' ? 'standard' : subtype;
}

/** Events are project clones whose charter mode is "event" (ADR 0057). */
export function isEvent(mode: CharterMode | undefined): boolean {
  return mode === 'event';
}

/** The left accent bar colour per mode (web's surfaceTypeAccent). */
export function modeAccent(mode: CharterMode): string {
  if (mode === 'event') return 'var(--type-accent-event)';
  if (mode === 'help-request') return 'var(--type-accent-help-request)';
  if (mode === 'productive') return 'var(--type-accent-project-production)';
  return mode === 'personal-service'
    ? 'var(--type-accent-project-personal-service)'
    : 'var(--type-accent-project-service)';
}

/** Personal-service projects take no demand/opposition signals (as on web). */
export function takesSignals(mode: CharterMode): boolean {
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
