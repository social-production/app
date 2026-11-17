/**
 * One symbol and one colour per kind of thing, shared by the map's pins and the feed cards'
 * icons (Map Redesign mockup, owner, 2026-11-17: "make feed card icons match the map, with
 * the bolder colours and these symbols"). Paths are drawn on a 24×24 grid with a round
 * 2.2 stroke in `currentColor`; colours are the `--kind-*` tokens in app.css.
 */
export type Kind = 'project' | 'service' | 'event' | 'request' | 'proposal' | 'thread' | 'post';

const stroke =
  'fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round"';

export const KIND_GLYPHS: Record<Kind, string> = {
  // A leaf: something growing.
  project: `<path d="M5 20c0-8 5-13 14-15-1 9-6 14-14 15Z" ${stroke}/><path d="M5 20 13 12" ${stroke}/>`,
  // A leaf in a hand's cup: a project that serves people.
  service: `<path d="M8 13c0-5 3-8 9-9-.6 5.6-3.8 8.6-9 9Z" ${stroke}/><path d="M3 15h4l3 2h5a2 2 0 0 1 0 4H8" ${stroke}/>`,
  event: `<rect x="4" y="5" width="16" height="15" rx="3" ${stroke}/><path d="M4 10h16M9 3v4M15 3v4" ${stroke}/>`,
  request: `<path d="M7 11V6.5a1.5 1.5 0 0 1 3 0V11M10 10V5a1.5 1.5 0 0 1 3 0v5M13 10V6a1.5 1.5 0 0 1 3 0v6c0 4-2.5 8-6.5 8S5 17 4.5 14l-1-3a1.5 1.5 0 0 1 2.8-1L7 12" ${stroke}/>`,
  proposal: `<path d="M9 18h6M10 21h4M12 3a6 6 0 0 0-3.5 10.9c.6.4 1 1.1 1 1.8V16h5v-.3c0-.7.4-1.4 1-1.8A6 6 0 0 0 12 3Z" ${stroke}/>`,
  thread: `<path d="M6 5.5h12A1.5 1.5 0 0 1 19.5 7v8a1.5 1.5 0 0 1-1.5 1.5h-8l-4 3v-3A1.5 1.5 0 0 1 4.5 15V7A1.5 1.5 0 0 1 6 5.5Z" ${stroke}/><path d="M8.5 9.5h7M8.5 12.5h4.5" ${stroke}/>`,
  post: `<path d="M5 19l1-4L16 5l3 3L9 18l-4 1Z" ${stroke}/><path d="M14 7l3 3" ${stroke}/>`,
};

/** The kind a charter mode (or a map marker's type and mode) shows as. */
export function kindOf(mode: string | null | undefined): Kind {
  switch (mode) {
    case 'event':
      return 'event';
    case 'help-request':
    case 'help_request':
      return 'request';
    case 'proposal':
      return 'proposal';
    case 'thread':
      return 'thread';
    case 'post':
      return 'post';
    case 'collective-service':
    case 'personal-service':
      return 'service';
    default:
      return 'project';
  }
}

/** The kind a map marker shows as: an activity takes its source's kind. */
export function kindOfMarker(item: {
  entityType?: string | null;
  activitySource?: string | null;
  projectMode?: string | null;
}): Kind {
  if (item.entityType === 'event') return 'event';
  if (item.entityType === 'help_request') return 'request';
  if (item.entityType === 'activity' && item.activitySource === 'event') return 'event';
  return kindOf(item.projectMode);
}

export const KIND_LABELS: Record<Kind, string> = {
  project: 'Project',
  service: 'Service',
  event: 'Event',
  request: 'Request',
  proposal: 'Proposal',
  thread: 'Thread',
  post: 'Post',
};

export const kindColor = (kind: Kind): string => `var(--kind-${kind})`;
