<script lang="ts">
  import type { Snippet } from 'svelte';

  import KindGlyph from '#lib/components/icons/KindGlyph.svelte';
  import { railKicker, type RailItem } from '#lib/domain/activityRail.ts';
  import { kindColor, kindOf } from '#lib/domain/kindGlyphs.ts';

  /** One row in Your activity (Screen Review §1, 2026-11-18): the kind's tile as on the feed
   * and the map, what it is and when, the title and its details; × clears it. */
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
  const kind = $derived(kindOf(p.item.projectMode));
</script>

<article
  class={`snapshot-row activity-row rail-item ${p.rowClass}`}
  class:activity-row-unseen={p.unseen}
>
  <button class="activity-open-button" type="button" onclick={p.onOpen}>
    <span class="rail-tile" style={`--tile: ${kindColor(kind)}`}
      ><KindGlyph {kind} size={18} /></span
    >
    <span class="rail-copy">
      <span class="card-kicker">{railKicker(p.item)}{p.time ? ` · ${p.time}` : ''}</span>
      <strong>{p.item.title}</strong>
      {#if p.item.body}<span class="card-body-preview">{p.item.body}</span>{/if}
      {#if p.item.meta}<span class="card-meta">{p.item.meta}</span>{/if}
      {#if p.item.countLabel}<span class="card-detail">{p.item.countLabel}</span>{/if}
    </span>
  </button>
  <button aria-label={p.dismissLabel} class="dismiss-card" type="button" onclick={p.onDismiss}>
    ×
  </button>
  {@render p.actions?.()}
</article>
