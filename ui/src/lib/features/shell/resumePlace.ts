/**
 * Reopen where you were (owner, 2026-11-24: "the app reloads after switching apps"). Phones
 * throw away a background web app to save memory; a home-screen app then restarts at its start
 * page. The page you were on and how far down you were are kept on this device, and a restart
 * at the start page within a few hours goes back there.
 */

import { goto } from '$app/navigation';

const KEY = 'sp.place';
/** How long a place is worth going back to. */
const FRESH_MS = 6 * 60 * 60 * 1000;
/** How long to keep trying to scroll back while the page's content loads. */
const SETTLE_MS = 4000;

interface Place {
  path: string;
  y: number;
  at: number;
}

function read(): Place | null {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return null;
    const place = JSON.parse(raw) as Partial<Place>;
    if (typeof place.path !== 'string' || typeof place.at !== 'number') return null;
    return { path: place.path, y: Number(place.y) || 0, at: place.at };
  } catch {
    return null;
  }
}

function save(): void {
  const { pathname, search, hash } = location;
  const place: Place = {
    path: `${pathname}${search}${hash}`,
    y: Math.round(scrollY),
    at: Date.now(),
  };
  try {
    localStorage.setItem(KEY, JSON.stringify(place));
  } catch {
    // Private mode or full storage: nothing to go back to next time.
  }
}

/** Scroll back once the page is tall enough, until the reader scrolls themselves. */
function scrollBack(y: number): void {
  if (y <= 0) return;
  const started = Date.now();
  let stopped = false;
  const stop = (): void => {
    stopped = true;
  };
  for (const event of ['wheel', 'touchstart', 'keydown'] as const)
    addEventListener(event, stop, { once: true, passive: true });
  const step = (): void => {
    if (stopped || Date.now() - started > SETTLE_MS) return;
    if (document.documentElement.scrollHeight - innerHeight >= y) {
      scrollTo({ top: y });
      return;
    }
    setTimeout(step, 150);
  };
  step();
}

/** A home-screen app: a browser tab keeps its own address when it reloads. */
function installed(): boolean {
  const ios = (navigator as Navigator & { standalone?: boolean }).standalone === true;
  return ios || matchMedia('(display-mode: standalone)').matches;
}

/** Start remembering, and go back to the last place if the app restarted at its start page.
 * Returns the cleanup. */
export function resumePlace(): () => void {
  const atStart = installed() && location.pathname === '/' && !location.search && !location.hash;
  const last = read();
  const away = last && last.path !== '/' && !last.path.startsWith('/onboarding');
  if (atStart && last && away && Date.now() - last.at < FRESH_MS) {
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- a path this app saved
    void goto(last.path, { replaceState: true }).then(() => scrollBack(last.y));
  }
  const onHide = (): void => {
    if (document.visibilityState === 'hidden') save();
  };
  document.addEventListener('visibilitychange', onHide);
  addEventListener('pagehide', save);
  return () => {
    document.removeEventListener('visibilitychange', onHide);
    removeEventListener('pagehide', save);
  };
}
