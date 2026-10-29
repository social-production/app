<script lang="ts">
  import { MediaQuery } from 'svelte/reactivity';

  import PrivateMark from './PrivateMark.svelte';
  import { tagHref, tagIcon } from './tagLinks.ts';

  import ScopeGlyph from '#lib/components/icons/ScopeGlyph.svelte';
  import ScopeSheet from '#lib/components/overlay/ScopeSheet.svelte';
  import { tagRef } from '#lib/domain/scopes.ts';

  /**
   * web's FeedCardTop with TagList (plain): tags left (3 on desktop, 1 on phones; "+N" opens
   * web's ScopeSheet with them all), the stage right, with web's PrivateMark beside it for
   * private events. Channel and community tags carry their icon and open their page; free
   * tags from before channels search for themselves.
   */
  let {
    tags,
    stage,
    isPrivate = false,
  }: { tags: string[]; stage: string; isPrivate?: boolean } = $props();

  const phone = new MediaQuery('(max-width: 760px)');
  let sheetOpen = $state(false);
  const refs = $derived(tags.map(tagRef));
  const visible = $derived(refs.slice(0, phone.current ? 1 : 3));
  const hidden = $derived(refs.length - visible.length);

  function openOverflow(event: MouseEvent): void {
    event.preventDefault();
    event.stopPropagation();
    sheetOpen = true;
  }

  import '#lib/components/styles/cards.css';
</script>

{#if tags.length > 0 || stage || isPrivate}
  <div class="feed-card-top">
    <div class="feed-card-tags">
      {#if tags.length > 0}
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
      {/if}
    </div>
    <div class="feed-card-end">
      {#if isPrivate}<PrivateMark />{/if}
      {#if stage}<span class="feed-card-stage">{stage}</span>{/if}
    </div>
  </div>
{/if}
