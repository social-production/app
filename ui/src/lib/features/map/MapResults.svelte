<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import SkeletonRows from '#lib/components/loading/SkeletonRows.svelte';
  import EmptyState from '#lib/components/empty/EmptyState.svelte';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import MapDiscoveryCard from '#lib/components/map/MapDiscoveryCard.svelte';
  import SpaceCard from '#lib/components/cards/SpaceCard.svelte';
  import type { Stance } from '#lib/domain/projects.ts';
  import { setSignal } from '#lib/services/projects.ts';
  import { requireViewer } from '#lib/stores/authNotice.svelte.ts';
  import { cardOf } from './mapCards.ts';
  import { markerHref } from './markerHref.ts';
  import type { MapPanelState } from './panelState.svelte.ts';

  /** web's MapPanel results list beside (or under) the map. */
  interface Props {
    s: MapPanelState;
    onHover: (id: string) => void;
    onLeave: () => void;
    onSelect: (id: string) => void;
    onClose: () => void;
    /** web's "Open as list", in the results' header on phones (ADR 0103). */
    onList: () => void;
  }

  let { s, onHover, onLeave, onSelect, onClose, onList }: Props = $props();

  const WHEN: Record<string, string> = {
    today: ' today',
    week: ' this week',
    month: ' this month',
    custom: ' in those dates',
  };
  // The empty map says what was searched and what to change (Scope Filters' empty states).
  const nothingText = $derived.by(() => {
    const where = s.radiusKm === 'global' ? 'anywhere' : `within ${s.radiusKm} km`;
    const when = WHEN[s.windowFilter] ?? '';
    const wider = s.radiusKm === 'global' ? '' : t('a wider distance');
    const longer = when ? t('a longer time') : '';
    const tryThis = [wider, longer].filter(Boolean).join(' or ');
    return `Nothing on the map ${where}${when}.${tryThis ? ` Try ${tryThis}.` : ''}`;
  });

  const isPast = (iso: string | null): boolean => !!iso && new Date(iso).getTime() < Date.now();

  async function signal(id: string, next: Stance | null): Promise<void> {
    if (!requireViewer('Sign in to support this project.')) {
      throw new Error('Sign in to support this project.');
    }
    await setSignal(id, next);
  }
</script>

<section class="list-panel" aria-label={t('Map results list')}>
  {#if s.loading && s.listItems.length === 0}
    <SkeletonRows count={3} kind="row" />
  {:else if s.errorMessage}
    <EmptyState icon="map-pin" title={t("The map couldn't load")} text={s.errorMessage} />
  {:else if !s.hasCenter}
    <EmptyState
      icon="map-pin"
      title={t('Choose a place')}
      text={t("Search a place or use your location to see what's near it.")}
    />
  {:else if s.listItems.length === 0}
    <EmptyState icon="map-pin" title={t('Nothing here yet')} text={nothingText} />
  {:else}
    {#if s.overlayFilters}
      <header class="results-head">
        <strong>{s.listItems.length} near here</strong>
        <button aria-label={t('Open as list')} class="results-list" type="button" onclick={onList}>
          <ToolbarIcon name="list" /><span>{t('List')}</span>
        </button>
      </header>
    {/if}
    <div class="card-list">
      {#each s.listItems as item (item.id)}
        {@const card = item.entityType === 'activity' ? null : cardOf(s.cards, item)}
        {#if card}
          <!-- The feed's own card (owner, 2026-11-25); hovering lights its pin. -->
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            class="map-feed-card"
            class:highlighted={s.highlightedId === item.id}
            onmouseenter={() => onHover(item.id)}
            onmouseleave={onLeave}
          >
            <SpaceCard {card} onSignal={(next) => signal(card.id, next)} />
          </div>
        {:else}
          <MapDiscoveryCard
            {item}
            href={markerHref(item)}
            isPast={!s.upcomingOnly && isPast(item.scheduledAt)}
            highlighted={s.highlightedId === item.id}
            {onHover}
            {onLeave}
            {onSelect}
            {onClose}
          />
        {/if}
      {/each}
    </div>
  {/if}
</section>
