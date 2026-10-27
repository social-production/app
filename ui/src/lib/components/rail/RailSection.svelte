<script lang="ts">
  import type { Snippet } from 'svelte';

  /** A collapsible rail section (web's `details.rail-section`), open at first when it has cards. */
  interface Props {
    title: string;
    count: number;
    hot: boolean;
    extraClass?: string;
    emptyTitle: string;
    emptyText: string;
    children?: Snippet;
  }

  let p: Props = $props();

  // svelte-ignore state_referenced_locally
  let open = $state(p.count > 0);

  $effect(() => {
    if (p.count === 0) open = false;
  });
</script>

<details
  class={`rail-section ${p.extraClass ?? ''}`}
  class:rail-section-collapsed={p.count === 0}
  bind:open
>
  <summary class="rail-section-summary" class:hot={p.hot}>
    <span class="summary-label">
      <span class="chevron" aria-hidden="true"></span>
      <span>{p.title}</span>
    </span>
  </summary>
  <div class:snapshot-scroll={p.count > 5} class="snapshot-stack">
    {#if p.count === 0}
      <div class="snapshot-row empty-hint">
        <strong>{p.emptyTitle}</strong>
        <span>{p.emptyText}</span>
      </div>
    {:else}
      {@render p.children?.()}
    {/if}
  </div>
</details>
