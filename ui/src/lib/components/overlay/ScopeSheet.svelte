<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { getContext } from 'svelte';

  import { SCOPE_PICTURES, type ScopePictureOf } from '#lib/domain/scopes.ts';
  import { tagHref, tagIcon, tagKindLabel } from '#lib/components/cards/tagLinks.ts';
  import ScopeGlyph from '#lib/components/icons/ScopeGlyph.svelte';
  import type { TagRef } from '#lib/domain/scopes.ts';

  import OverlaySheet from './OverlaySheet.svelte';

  import '#lib/components/styles/scopes.css';

  /** web's ScopeSheet: every tag of a card (its "+N"), searchable, each opening its page. */
  interface Props {
    open: boolean;
    tags: TagRef[];
    onClose: () => void;
  }

  let { open, tags, onClose }: Props = $props();

  let query = $state('');
  $effect(() => {
    if (!open) query = '';
  });

  const needle = $derived(query.trim().toLowerCase());
  const filtered = $derived(
    [...tags]
      .sort((a, b) => a.label.localeCompare(b.label, undefined, { sensitivity: 'base' }))
      .filter((tag) => !needle || tag.label.toLowerCase().includes(needle)),
  );

  const pictureOf = getContext<ScopePictureOf | undefined>(SCOPE_PICTURES);
</script>

<OverlaySheet {open} title="Channels & communities" rootClass="scope-sheet" {onClose}>
  {#snippet toolbar()}
    <div class="sheet-search">
      <label class="sr-only" for="scope-sheet-search">{t('Search channels and communities')}</label>
      <input
        id="scope-sheet-search"
        bind:value={query}
        placeholder={t('Search channels and communities')}
        type="search"
      />
    </div>
  {/snippet}
  <div class="scope-list">
    {#if filtered.length === 0}
      <p class="empty-row">
        {query.trim() ? t('No matches.') : t('No tagged channels or communities.')}
      </p>
    {:else}
      {#each filtered as tag (tag.tag)}
        {@const icon = tagIcon(tag)}
        <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- tagHref resolves -->
        <a class="scope-row" href={tagHref(tag)} onclick={onClose}>
          <span class="scope-icon">
            {#if icon}<ScopeGlyph {icon} size="md" imageUrl={pictureOf?.(tag.slug) ?? null} />{/if}
          </span>
          <span class="scope-copy">
            <strong>{t(tag.label)}</strong>
            <span>{t(tagKindLabel(tag))}</span>
          </span>
        </a>
      {/each}
    {/if}
  </div>
</OverlaySheet>
