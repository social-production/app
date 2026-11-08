import { untrack } from 'svelte';
import { mapFocus } from '#lib/stores/mapFocus.svelte.ts';

type Rails = { left: boolean; right: boolean };

/**
 * web's AppShell map toggling (`toggleMapPanel`, `closeMapPanel`): opening closes the rails
 * (desktop remembers them and restores them on close); a route change closes the panel; a
 * page's `openPlaceOnMap` opens it (ADR 0085).
 */
export class MapToggle {
  open = $state(false);
  private before: Rails = { left: false, right: false };

  constructor(
    private readonly rails: () => Rails & { compact: boolean },
    private readonly setRails: (rails: Rails) => void,
    route: () => { path: string; mapPage: boolean },
  ) {
    // Made while the shell starts, so these effects live as long as it does.
    $effect(() => {
      void route().path;
      untrack(() => this.close());
    });
    $effect(() => {
      if (mapFocus.request && !route().mapPage) untrack(() => !this.open && this.toggle());
    });
  }

  toggle(): void {
    if (this.open) return this.close();
    const { left, right, compact } = this.rails();
    if (!compact) this.before = { left, right };
    this.open = true;
    mapFocus.open = true;
    this.setRails({ left: false, right: false });
  }

  close(): void {
    if (!this.open) return;
    this.open = false;
    mapFocus.open = false;
    if (!this.rails().compact) this.setRails(this.before);
  }
}
