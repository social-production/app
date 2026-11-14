<script lang="ts">
  import type { Snippet } from 'svelte';
  import { MediaQuery } from 'svelte/reactivity';

  import PrivateMark from './PrivateMark.svelte';
  import { tagHref, tagIcon } from './tagLinks.ts';

  import SurfaceTypeLabel from '#lib/components/rail/SurfaceTypeLabel.svelte';
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
    children,
  }: {
    tags: string[];
    stage: string;
    isPrivate?: boolean;
    mode?: CharterMode | null;
    createdAt?: number | null;
    updatedAt?: number | null;
    children?: Snippet;
  } = $props();

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
</script>

{#snippet tagList()}
  <div class="tag-list plain">
    {#each visible as tag, index (tag.tag)}
      {@const icon = tagIcon(tag)}
      {#if index > 0}<span class="tag-sep" aria-hidden="true">,</span>{/if}
      <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- tagHref resolves -->
      <a class="tag-plain" href={tagHref(tag)}
        >{#if icon}<ScopeGlyph {icon} />{/if}<span>{tag.label}</span></a
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
      >{relativeCompact(activityInstant(createdAt, updatedAt))}</span
    >
  {/if}
{/snippet}

{#if lead && mode}
  <!-- A feed row (ADR 0102): the type's glyph, the channel in bold, then type · phase · clock. -->
  <div class="feed-card-top feed-lead">
    <span class="feed-lead-glyph"><SurfaceTypeLabel projectMode={mode} /></span>
    <div class="feed-lead-copy">
      {#if tags.length > 0}
        <div class="feed-lead-tags">{@render tagList()}</div>
      {/if}
      <div class="feed-lead-meta" class:alone={tags.length === 0}>
        <SurfaceTypeLabel projectMode={mode} />
        {#if stage}<span class="feed-card-stage">{stage}</span>{/if}
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
      {#if stage}<span class="feed-card-stage">{stage}</span>{/if}
      {@render children?.()}
    </div>
  </div>
{/if}
