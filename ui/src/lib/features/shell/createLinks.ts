import type { CreateLink } from '#lib/components/shell/CreateFab.svelte';

import { resolve } from '$app/paths';

/** web's createMenuItems, in web's order. Since ADR 0059 a public project or event starts
 * as a Proposal with no type; only private events are created as events. */
export const createLinks: CreateLink[] = [
  {
    href: resolve('/create/post'),
    label: 'Post',
    description: 'Share to your personal feed',
    icon: 'post',
  },
  {
    href: resolve('/create/thread'),
    label: 'Thread',
    description: 'Start a discussion thread',
    icon: 'thread',
  },
  {
    href: resolve('/create/help-request'),
    label: 'Request',
    description: 'Ask for help with roles and schedule',
    icon: 'help-request',
  },
  {
    href: resolve('/create/project'),
    label: 'Proposal',
    description: 'Propose a project, a service or a public event',
    icon: 'project',
  },
  {
    href: resolve('/create/event'),
    label: 'Private event',
    description: 'Invite people to a gathering',
    icon: 'event',
  },
];

/** web's createSurfaceLinks. */
export const surfaceLinks: CreateLink[] = [
  {
    href: resolve('/create/community'),
    label: 'Community',
    description: 'Coordination space around shared activity',
    icon: 'community',
  },
  {
    href: resolve('/create/channel'),
    label: 'Channel',
    description: 'Topic-based discovery surface',
    icon: 'channel',
  },
];
