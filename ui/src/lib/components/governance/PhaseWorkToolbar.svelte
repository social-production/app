<script lang="ts">
  import type { Snippet } from 'svelte';

  import '#lib/components/styles/governance.css';

  /**
   * web's PhaseWorkToolbar: a phase's work actions ("Add plan") and the Phase ▾ menu, moved into
   * the Take part dock (`#detail-participation-actions`). The phase menu portals itself into
   * `#phase-nav-end`. ADR 0099: the dock is one capsule and lays the actions out itself
   * (`styles/ux/dock.css`), so web's row-packing script is gone.
   */
  let { children, governance }: { children?: Snippet; governance?: Snippet } = $props();

  let node = $state<HTMLDivElement>();

  $effect(() => {
    const group = node;
    if (!group) return;
    const target = document.getElementById('detail-participation-actions');
    if (target && group.parentElement !== target) target.appendChild(group);
    return () => group.remove();
  });
</script>

<div class="phase-action-group" bind:this={node}>
  <div class="adds">{@render children?.()}</div>
  <div class="governance-row">{@render governance?.()}</div>
  <div id="phase-nav-vote" class="phase-action-slot"></div>
  <div class="phase-nav-bar">
    <div id="phase-nav-start" class="phase-action-slot"></div>
    <div id="phase-nav-end" class="phase-action-slot"></div>
  </div>
</div>
