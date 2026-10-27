<script lang="ts">
  import type { Snippet } from 'svelte';

  import { railKicker, type RailItem } from '#lib/domain/activityRail.ts';
  import { modeAccent } from '#lib/domain/projects.ts';

  import SurfaceTypeLabel from './SurfaceTypeLabel.svelte';

  /** One card in a rail section (web's `.snapshot-row.activity-row`). */
  interface Props {
    item: RailItem;
    unseen: boolean;
    rowClass: string;
    dismissLabel: string;
    time: string;
    onOpen: () => void;
    onDismiss: () => void;
    actions?: Snippet;
  }

  let p: Props = $props();
</script>

<article
  class={`snapshot-row activity-row ${p.rowClass}`}
  class:activity-row-unseen={p.unseen}
  style={`--row-accent: ${modeAccent(p.item.projectMode)};`}
>
  <button aria-label={p.dismissLabel} class="dismiss-card" type="button" onclick={p.onDismiss}>
    ×
  </button>
  <button class="activity-open-button" type="button" onclick={p.onOpen}>
    <div class="activity-topline">
      <SurfaceTypeLabel projectMode={p.item.projectMode} />
      <span class="snapshot-time">{p.time}</span>
    </div>
    <span class="card-kicker">{railKicker(p.item)}</span>
    <strong>{p.item.title}</strong>
    {#if p.item.meta}
      <span class="card-meta">{p.item.meta}</span>
    {/if}
    {#if p.item.countLabel}
      <span class="card-detail">{p.item.countLabel}</span>
    {/if}
  </button>
  {@render p.actions?.()}
</article>
