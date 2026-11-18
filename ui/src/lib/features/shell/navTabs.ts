import { t } from '#lib/i18n/index.ts';
import type { ResolvedPathname } from '$app/types';

import { resolve } from '$app/paths';

export interface NavTab {
  href: ResolvedPathname;
  label: string;
  icon: string;
  badge?: number;
}

/**
 * The bottom nav's tabs. Someone browsing signed out gets only Home: Notifications and
 * Messages need an account, so they are left out rather than leading to sign-up.
 */
export function navTabs(guest: boolean, notifications: number, messages: number): NavTab[] {
  const home: NavTab = { href: resolve('/'), label: t('Home'), icon: 'home' };
  if (guest) return [home];
  return [
    home,
    {
      href: resolve('/notifications'),
      label: t('Notifications'),
      icon: 'bell',
      badge: notifications,
    },
    { href: resolve('/messages'), label: t('Messages'), icon: 'send', badge: messages },
  ];
}
