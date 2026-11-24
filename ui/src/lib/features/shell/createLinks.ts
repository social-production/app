import { t } from '#lib/i18n/index.ts';
import type { CreateLink } from '#lib/components/shell/CreateFab.svelte';

import { resolve } from '$app/paths';

/** web's createMenuItems, in web's order. Since ADR 0059 a public project or event starts
 * as a Proposal with no type; only private events are created as events. */
export const createLinks: CreateLink[] = [
  {
    href: resolve('/create/post'),
    label: t('Post'),
    description: t('Share to your personal feed'),
    icon: 'post',
  },
  {
    href: resolve('/create/thread'),
    label: t('Thread'),
    description: t('Start a discussion thread'),
    icon: 'thread',
  },
  {
    href: resolve('/create/help-request'),
    label: t('Request'),
    description: t('Ask for help with roles and schedule'),
    icon: 'help-request',
  },
  {
    href: resolve('/create/project'),
    label: t('Proposal'),
    description: t('Propose a project, a service or a public event'),
    icon: 'proposal',
    badge: 'NEW',
  },
  {
    href: resolve('/create/event'),
    label: t('Private event'),
    description: t('Invite people to a gathering'),
    icon: 'event',
  },
  {
    href: resolve('/create/meetup'),
    label: t('Private meetup'),
    description: t('Meet regularly or whenever, like pickup games'),
    icon: 'event',
  },
];

/** web's createSurfaceLinks. */
export const surfaceLinks: CreateLink[] = [
  {
    href: resolve('/create/community'),
    label: t('Community'),
    description: t('Coordination space around shared activity'),
    icon: 'community',
  },
  {
    href: resolve('/create/channel'),
    label: t('Channel'),
    description: t('Topic-based discovery surface'),
    icon: 'channel',
  },
];
