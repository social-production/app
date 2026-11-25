<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { getContext, type Snippet } from 'svelte';

  import KindGlyph from '#lib/components/icons/KindGlyph.svelte';
  import { railKicker, type RailItem } from '#lib/domain/activityRail.ts';
  import { KIND_LABELS, kindColor, kindOfSpace } from '#lib/domain/kindGlyphs.ts';
  import { railClock } from '#lib/domain/railDays.ts';
  import { SPACE_PICTURES, type SpacePictureOf } from '#lib/domain/scopes.ts';

  /** One card in Your activity (Rail and Group Chat mockup §1, owner 2026-11-25): the space's
   * picture with its kind on the corner, what it is and whose, the title and its details, an
   * activity's start in a chip and a vote's progress as a bar; × clears it. */
  interface Props {
    item: RailItem;
    unseen: boolean;
    rowClass: string;
    dismissLabel: string;
    onOpen: () => void;
    onDismiss: () => void;
    actions?: Snippet;
  }

  let p: Props = $props();
  const kind = $derived(kindOfSpace(p.item.projectMode, p.item.subtype));
  const pictureOf = getContext<SpacePictureOf | undefined>(SPACE_PICTURES);
  const picture = $derived(p.item.picture ? (pictureOf?.(p.item.picture) ?? null) : null);
  const activity = $derived(p.item.kind === 'project');
  const kicker = $derived(
    activity && p.item.spaceTitle
      ? `${t(KIND_LABELS[kind])} · ${p.item.spaceTitle}`
      : railKicker(p.item),
  );
  const details = $derived(
    [p.item.meta, activity ? p.item.countLabel && t(p.item.countLabel) : '']
      .filter(Boolean)
      .join(' · '),
  );
  const clock = $derived(activity ? railClock(p.item.createdAt) : null);
  const share = $derived(
    p.item.progress && p.item.progress.required > 0
      ? Math.min(100, Math.round((p.item.progress.votes / p.item.progress.required) * 100))
      : null,
  );
</script>

<article
  class={`activity-row rail-item rail-card ${p.rowClass}`}
  class:activity-row-unseen={p.unseen}
>
  <button class="activity-open-button" type="button" onclick={p.onOpen}>
    <span class="rail-tile" class:has-picture={!!picture} style={`--tile: ${kindColor(kind)}`}>
      {#if picture}
        <img src={picture} alt="" />
        <span class="rail-tile-kind"><KindGlyph {kind} size={11} /></span>
      {:else}
        <KindGlyph {kind} size={20} />
      {/if}
    </span>
    <span class="rail-copy">
      <span class="card-kicker">{kicker}</span>
      <strong>{p.item.title}</strong>
      {#if p.item.body}<span class="card-body-preview">{p.item.body}</span>{/if}
      {#if details}<span class="card-meta">{details}</span>{/if}
      {#if share !== null && p.item.progress}
        <span class="rail-bar" aria-hidden="true"><span style={`width: ${share}%`}></span></span>
        <span class="card-detail rail-vote-count"
          >{t('{n} of {m} votes', {
            n: String(p.item.progress.votes),
            m: String(p.item.progress.required),
          })}</span
        >
      {:else if !activity && p.item.countLabel}
        <span class="card-detail">{t(p.item.countLabel)}</span>
      {/if}
    </span>
    {#if clock}
      <span class="rail-clock"
        >{clock.time}{#if clock.half}<small>{clock.half}</small>{/if}</span
      >
    {/if}
  </button>
  <button aria-label={p.dismissLabel} class="dismiss-card" type="button" onclick={p.onDismiss}>
    ×
  </button>
  {@render p.actions?.()}
</article>
