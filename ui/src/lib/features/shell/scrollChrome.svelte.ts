import { untrack } from 'svelte';

/** Pages that read as a feed: home, the platform, profiles, channels and communities. */
export function isFeedSurface(path: string): boolean {
  return (
    ['/', '/platform', '/profile'].includes(path) ||
    /^\/(profile|channels|communities)\/[^/]+$/.test(path)
  );
}

/** How far (px) you scroll one way before the bars move; small jitters don't flicker them. */
const TRAVEL = 24;
/** Near the top the bars always show. */
const TOP = 60;

/**
 * On a phone feed the top bar and bottom nav slide away while you scroll down and come back
 * as soon as you scroll up, as on other social apps. A route change shows them again.
 */
export class ScrollChrome {
  hidden = $state(false);

  constructor(route: () => { on: boolean; path: string }) {
    // Made while the shell starts, so the effect lives as long as it does.
    $effect(() => {
      const { on } = route();
      untrack(() => (this.hidden = false));
      if (!on) return;
      let last = window.scrollY;
      let travel = 0;
      let frame = 0;
      const update = (): void => {
        frame = 0;
        const y = Math.max(0, window.scrollY);
        const delta = y - last;
        last = y;
        if (y < TOP) {
          travel = 0;
          this.hidden = false;
          return;
        }
        travel = Math.sign(delta) === Math.sign(travel) ? travel + delta : delta;
        if (travel > TRAVEL) this.hidden = true;
        else if (travel < -TRAVEL) this.hidden = false;
      };
      const onScroll = (): void => {
        if (!frame) frame = requestAnimationFrame(update);
      };
      window.addEventListener('scroll', onScroll, { passive: true });
      return () => {
        window.removeEventListener('scroll', onScroll);
        cancelAnimationFrame(frame);
      };
    });
  }
}
