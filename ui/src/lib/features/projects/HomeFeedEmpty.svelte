<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import { resolve } from '$app/paths';
  import EmptyState from '#lib/components/empty/EmptyState.svelte';
  import type { FeedScope } from '#lib/domain/homeFeed.ts';

  /**
   * What Home, Global and Region say when they have nothing to show (Scope Filters mockups §2),
   * most specific first: a failed load, no place for Region, filters that leave nothing, a
   * search with no results, then each tab's own empty feed.
   */
  interface Props {
    error?: string | undefined;
    scope: FeedScope;
    hasPlace: boolean;
    narrowed: boolean;
    search: string;
    onChoosePlace: () => void;
    onClear: () => void;
  }

  let { error, scope, hasPlace, narrowed, search, onChoosePlace, onClear }: Props = $props();
</script>

{#if error}
  <EmptyState icon="info" title={t("The feed didn't load")} text={error} />
{:else if scope === 'region' && !hasPlace}
  <EmptyState
    icon="map-pin"
    title={t('Pick a place')}
    text={t("Choose a place or use your location to see what's near.")}
  >
    <button class="empty-action" type="button" onclick={onChoosePlace}>{t('Choose a place')}</button
    >
  </EmptyState>
{:else if narrowed}
  <EmptyState
    icon="filter"
    title={t('Nothing matches these filters')}
    text={t('Try a longer time or another type.')}
  >
    <button class="empty-action" type="button" onclick={onClear}>{t('Clear filters')}</button>
  </EmptyState>
{:else if search}
  <EmptyState
    icon="search"
    title={t('No results for “{query}”', { query: search })}
    text={t('Check the spelling or try fewer words.')}
  />
{:else if scope === 'home'}
  <EmptyState
    icon="home"
    title={t('Your Home is quiet')}
    text={t('Join channels or follow people to fill it.')}
  >
    <a class="empty-action" href={resolve('/search')}>{t('Find channels')}</a>
  </EmptyState>
{:else if scope === 'region'}
  <EmptyState
    icon="map-pin"
    title={t('Nothing near here yet')}
    text={t('Try a wider distance, or start something nearby.')}
  >
    <button class="empty-action" type="button" onclick={onChoosePlace}
      >{t('Change distance')}</button
    >
  </EmptyState>
{:else}
  <EmptyState
    icon="globe"
    title={t('Nothing here yet')}
    text={t('Start a project, event or thread and it shows up here.')}
  >
    <a class="empty-action primary" href={resolve('/create/thread')}>{t('Start a thread')}</a>
  </EmptyState>
{/if}
