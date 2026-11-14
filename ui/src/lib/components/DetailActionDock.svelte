<script lang="ts">
  import type { Snippet } from 'svelte';

  import BackButton from '#lib/components/BackButton.svelte';
  import '#lib/components/styles/detail.css';
  import '#lib/components/styles/ux/dock.css';

  /**
   * web's DetailActionDock: a detail page's action bar. While `active` it moves into the
   * app shell so it can stick to the bottom (phones) or float over the column (wide
   * desktop), and publishes its height as --detail-action-dock-height for page padding.
   * ADR 0099: it floats in the nav capsule's style and takes the nav's place: a round back
   * button, one capsule of actions (`children`), and above them the `vote` capsule.
   */
  let { active, vote, children }: { active: boolean; vote?: Snippet; children: Snippet } = $props();

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
    <BackButton />
    <div class="dock-capsule">{@render children()}</div>
  </div>
</div>
