import { t } from '#lib/i18n/index.ts';

/** The About page's "Get the app" rows (ADR 0115). Desktop files come from the rolling
 * `desktop-latest` prerelease that `desktop.yml` publishes from main. */
const RELEASE = 'https://github.com/social-production/app/releases/download/desktop-latest';

export type Platform = 'windows' | 'macos' | 'linux' | 'android' | 'ios';

export interface AppOption {
  id: Platform;
  name: string;
  note: string;
  /** A download link, or none when the app installs from the browser. */
  href?: string;
}

export function appOptions(): AppOption[] {
  return [
    {
      id: 'windows',
      name: t('Windows'),
      note: t('Windows 10 or later.'),
      href: `${RELEASE}/social-production-windows.exe`,
    },
    {
      id: 'macos',
      name: t('Mac'),
      note: t('Apple silicon (M1 or later).'),
      href: `${RELEASE}/social-production-macos.tar.gz`,
    },
    {
      id: 'linux',
      name: t('Linux'),
      note: t('Needs WebKitGTK 4.1.'),
      href: `${RELEASE}/social-production-linux.tar.gz`,
    },
    {
      id: 'android',
      name: t('Android'),
      note: t('In Chrome, open the ⋮ menu and tap Install app.'),
    },
    {
      id: 'ios',
      name: t('iPhone and iPad'),
      note: t('In Safari, tap Share, then Add to Home Screen.'),
    },
  ];
}

/** The visitor's own device, from the browser's user agent, so it can be listed first. */
export function detectPlatform(userAgent: string, touchPoints = 0): Platform | null {
  if (/android/i.test(userAgent)) return 'android';
  if (/iphone|ipad|ipod/i.test(userAgent)) return 'ios';
  // iPadOS reports itself as a Mac; only the touch screen tells them apart.
  if (/macintosh/i.test(userAgent)) return touchPoints > 1 ? 'ios' : 'macos';
  if (/windows/i.test(userAgent)) return 'windows';
  if (/linux|x11/i.test(userAgent)) return 'linux';
  return null;
}

export function ownFirst(options: AppOption[], own: Platform | null): AppOption[] {
  const mine = options.filter((o) => o.id === own);
  return [...mine, ...options.filter((o) => o.id !== own)];
}
