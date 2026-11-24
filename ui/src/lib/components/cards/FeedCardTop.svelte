<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { getContext } from 'svelte';

  import {
    SCOPE_PICTURES,
    SPACE_PICTURES,
    type ScopePictureOf,
    type SpacePictureOf,
  } from '#lib/domain/scopes.ts';
  import type { Snippet } from 'svelte';
  import { MediaQuery } from 'svelte/reactivity';

  import PrivateMark from './PrivateMark.svelte';
  import { tagHref, tagIcon } from './tagLinks.ts';

  import SurfaceTypeLabel from '#lib/components/rail/SurfaceTypeLabel.svelte';
  import { isNetworkVote } from '#lib/domain/board.ts';
  import { tagRef } from '#lib/domain/scopes.ts';
  import type { CharterMode } from '#lib/domain/projects.ts';
  import { activityInstant, describeUpdate, relativeCompact } from '#lib/domain/time.ts';
  import ScopeGlyph from '#lib/components/icons/ScopeGlyph.svelte';
  import ScopeSheet from '#lib/components/overlay/ScopeSheet.svelte';

  /**
   * web's FeedCardTop with TagList (plain): tags left (3 on desktop, 1 on phones; "+N" opens
   * web's ScopeSheet with them all), the stage right, with web's PrivateMark beside it for
   * private events. Channel and community tags carry their icon and open their page; free
   * tags from before channels search for themselves. web's slot (the ⋮ report menu) sits
   * after the stage. Feed rows (ADR 0097) lead with the type as icon + label and end the
   * left side with the clock; since ADR 0102 they lead with the channel beside the type's
   * glyph, and the type, phase and clock sit underneath.
   */
  let {
    tags,
    stage,
    isPrivate = false,
    mode = null,
    createdAt = null,
    updatedAt = null,
    distanceKm = null,
    picture = null,
    onPicture,
    children,
  }: {
    tags: string[];
    stage: string;
    isPrivate?: boolean;
    mode?: CharterMode | null;
    createdAt?: number | null;
    updatedAt?: number | null;
    /** km away, before the clock (Region cards). */
    distanceKm?: number | null;
    /** A project's or event's picture hash (ADR 0117): the row's tile instead of the type. */
    picture?: string | null;
    /** A detail page's header: the picture opens full screen. */
    onPicture?: (() => void) | undefined;
    children?: Snippet;
  } = $props();
  const spacePicture = getContext<SpacePictureOf | undefined>(SPACE_PICTURES);
  const pictureUrl = $derived(picture ? (spacePicture?.(picture) ?? null) : null);
  // A #platform project takes a network vote (ADR 0095); threads and posts take none.
  const network = $derived(
    isNetworkVote(tags) && mode !== null && mode !== 'thread' && mode !== 'post',
  );

  const phone = new MediaQuery('(max-width: 760px)');
  let sheetOpen = $state(false);
  const refs = $derived(tags.map(tagRef));
  // A feed row leads with its channel alone; the rest are behind "+N".
  const lead = $derived(createdAt !== null);
  const visible = $derived(refs.slice(0, phone.current || lead ? 1 : 3));
  const hidden = $derived(refs.length - visible.length);

  function openOverflow(event: MouseEvent): void {
    event.preventDefault();
    event.stopPropagation();
    sheetOpen = true;
  }

  import '#lib/components/styles/cards.css';

  // Feed rows keep the bold channel name alone; other tag lists show its picture (owner).
  const pictureOf = getContext<ScopePictureOf | undefined>(SCOPE_PICTURES);
</script>

{#snippet tagList()}
  <div class="tag-list plain">
    {#each visible as tag, index (tag.tag)}
      {@const icon = tagIcon(tag)}
      {#if index > 0}<span class="tag-sep" aria-hidden="true">,</span>{/if}
      <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- tagHref resolves -->
      <a class="tag-plain" href={tagHref(tag)}
        >{#if icon}<ScopeGlyph
            {icon}
            imageUrl={lead ? null : (pictureOf?.(tag.slug) ?? null)}
          />{/if}<span>{t(tag.label)}</span></a
      >
    {/each}
    {#if hidden > 0}
      <span class="tag-sep" aria-hidden="true">,</span>
      <button
        type="button"
        class="tag-overflow-btn"
        aria-haspopup="dialog"
        aria-expanded={sheetOpen}
        aria-label={`Show ${hidden} more tags`}
        onclick={openOverflow}>+{hidden}</button
      >
    {/if}
  </div>
  <ScopeSheet open={sheetOpen} tags={refs} onClose={() => (sheetOpen = false)} />
{/snippet}

{#snippet clock()}
  {#if createdAt !== null}
    <span class="feed-card-time" title={describeUpdate(createdAt, updatedAt)}
      >{distanceKm !== null && distanceKm !== undefined
        ? `${distanceKm.toFixed(1)} km · `
        : ''}{relativeCompact(activityInstant(createdAt, updatedAt))}</span
    >
  {/if}
{/snippet}

{#if lead && mode}
  <!-- A feed row (ADR 0102): the type's glyph, the channel in bold, then type · phase · clock. -->
  <div class="feed-card-top feed-lead">
    {#if pictureUrl && onPicture}
      <button
        class="feed-lead-glyph"
        type="button"
        aria-label={t('View picture')}
        onclick={onPicture}><img class="feed-lead-picture" src={pictureUrl} alt="" /></button
      >
    {:else}
      <span class="feed-lead-glyph"
        >{#if pictureUrl}<img
            class="feed-lead-picture"
            src={pictureUrl}
            alt=""
          />{:else}<SurfaceTypeLabel projectMode={mode} feedGlyph />{/if}</span
      >
    {/if}
    <div class="feed-lead-copy">
      {#if tags.length > 0}
        <div class="feed-lead-tags">{@render tagList()}</div>
      {/if}
      <div class="feed-lead-meta" class:alone={tags.length === 0}>
        <SurfaceTypeLabel projectMode={mode} />
        <!-- A proposal in its Proposal phase would read "Proposal · Proposal". -->
        {#if network}
          <span class="feed-card-stage network-vote">{t('Network vote')}</span>
        {:else if stage && !(mode === 'proposal' && stage === 'Proposal')}
          <span class="feed-card-stage">{t(stage)}</span>
        {/if}
        {@render clock()}
      </div>
    </div>
    <div class="feed-card-end">
      {#if isPrivate}<PrivateMark />{/if}
      {@render children?.()}
    </div>
  </div>
{:else if tags.length > 0 || stage || isPrivate || children || mode}
  <div class="feed-card-top">
    <div class="feed-card-tags">
      {#if mode}<SurfaceTypeLabel projectMode={mode} />{/if}
      {#if tags.length > 0}{@render tagList()}{/if}
      {@render clock()}
    </div>
    <div class="feed-card-end">
      {#if isPrivate}<PrivateMark />{/if}
      {#if stage}<span class="feed-card-stage">{t(stage)}</span>{/if}
      {@render children?.()}
    </div>
  </div>
{/if}
