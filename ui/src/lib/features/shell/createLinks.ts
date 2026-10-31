import type { CreateLink } from '#lib/components/shell/CreateFab.svelte';

import { resolve } from '$app/paths';

/** web's createMenuItems: the content the app has so far, in web's order. */
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
    label: 'Help request',
    description: 'Ask for help with roles and schedule',
    icon: 'help-request',
  },
  {
    href: resolve('/create/project'),
    label: 'Project',
    description: 'Start a productive or service project',
    icon: 'project',
  },
  {
    href: resolve('/create/event'),
    label: 'Event',
    description: 'Plan a scheduled gathering',
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
