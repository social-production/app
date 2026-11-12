<script lang="ts">
  import type { Snippet } from 'svelte';

  import { resolve } from '$app/paths';

  import '#lib/components/styles/detail.css';
  import '#lib/components/styles/ux/dock.css';

  /**
   * web's DetailActionDock: a detail page's action bar. While `active` it moves into the
   * app shell so it can stick to the bottom (phones) or float over the column (wide
   * desktop), and publishes its height as --detail-action-dock-height for page padding.
   * ADR 0098: it floats in the nav capsule's style and takes the nav's place: a round back
   * button, one capsule of actions (`children`), and above them the `vote` capsule.
   */
  let { active, vote, children }: { active: boolean; vote?: Snippet; children: Snippet } = $props();

  function back(): void {
    if (history.length > 1) history.back();
    else location.assign(resolve('/'));
  }

  let node = $state<HTMLDivElement>();

  function columnSpan(): number {
    const shell = document.querySelector('.shell');
    const styles = shell instanceof HTMLElement ? getComputedStyle(shell) : null;
    const width = (name: string): number =>
      styles ? parseFloat(styles.getPropertyValue(name)) || 0 : 0;
    return window.innerWidth - width('--left-width') - width('--right-width');
  }

  function sync(dock: HTMLDivElement): void {
    const floating = window.matchMedia('(min-width: 1081px)').matches && columnSpan() > 720;
    dock.classList.toggle('is-pad', floating);
    if (!active) return;
    const hidden = dock.hidden || getComputedStyle(dock).display === 'none';
    const height = hidden ? 0 : dock.offsetHeight + (floating ? 12 : 0);
    document.documentElement.style.setProperty('--detail-action-dock-height', `${height}px`);
  }

  $effect(() => {
    const dock = node;
    if (!dock) return;
    // A marker holds the dock's place. Svelte removes a block's DOM before running its
    // cleanups, so a dock moved into the shell must be dropped, not re-homed, once the
    // marker is gone (otherwise it outlives the block, e.g. on the Chat tab).
    const marker = document.createComment('detail-action-dock');
    dock.before(marker);
    const shell = document.querySelector('.shell');
    if (active && shell instanceof HTMLElement) shell.appendChild(dock);
    sync(dock);
    const observer = new ResizeObserver(() => sync(dock));
    observer.observe(dock);
    const onResize = (): void => sync(dock);
    window.addEventListener('resize', onResize);
    return () => {
      observer.disconnect();
      window.removeEventListener('resize', onResize);
      if (marker.isConnected) marker.replaceWith(dock);
      else dock.remove();
      document.documentElement.style.setProperty('--detail-action-dock-height', '0px');
    };
  });
</script>

<div class="detail-action-dock" bind:this={node}>
  {#if vote}<div class="dock-vote">{@render vote()}</div>{/if}
  <div class="dock-row">
    <button class="dock-back" type="button" aria-label="Back" title="Back" onclick={back}>
      <svg aria-hidden="true" viewBox="0 0 24 24"
        ><path
          d="M14.5 5.5 8 12l6.5 6.5"
          fill="none"
          stroke="currentColor"
          stroke-width="2.2"
          stroke-linecap="round"
          stroke-linejoin="round"
        /></svg
      >
    </button>
    <div class="dock-capsule">{@render children()}</div>
  </div>
</div>
