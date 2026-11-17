<script lang="ts">
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import SegmentedChoice from '#lib/components/SegmentedChoice.svelte';
  import type { FeedType } from '#lib/domain/feedTypes.ts';
  import type { FeedScope, HomeAudience } from '#lib/domain/homeFeed.ts';
  import { radiusPresetOptions } from '#lib/domain/mapRadius.ts';

  import FeedFilterSheet from './FeedFilterSheet.svelte';
  import type { HomeFeedState } from './homeFeedState.svelte.ts';
  import RegionControls from './RegionControls.svelte';

  /**
   * The top of the public feed (ADR 0102): Home, Global and Region as word tabs, and one
   * button opening a sheet with web's other four menus (audience, filter, sort, window)
   * under their names. Region keeps its place, radius and online controls under the tabs.
   */
  let {
    feed,
    signedIn,
    guest = false,
    filter = $bindable(),
    open = $bindable(false),
  }: {
    feed: HomeFeedState;
    signedIn: boolean;
    guest?: boolean;
    filter: FeedType;
    open?: boolean;
  } = $props();

  const allScopes: { value: FeedScope; label: string }[] = [
    { value: 'home', label: 'Home' },
    { value: 'global', label: 'Global' },
    { value: 'region', label: 'Region' },
  ];
  // Home is the feed of who you follow: someone signed out has none, so it isn't offered.
  const scopes = $derived(guest ? allScopes.filter((s) => s.value !== 'home') : allScopes);
  const audiences: { value: HomeAudience; label: string }[] = [
    { value: 'following', label: 'Following only' },
    { value: 'popular', label: 'Following + popular' },
  ];
  const radiusLabel = $derived(
    radiusPresetOptions.find((o) => o.value === feed.region.radius)?.label ??
      `${feed.region.radius} km`,
  );
  const scopeLabel = $derived(scopes.find((s) => s.value === feed.shownScope)?.label ?? 'Global');
  // The dot: something narrower than the feed's widest view is on.
  const narrowed = $derived(filter !== 'all' || feed.window !== 'all');
</script>

<header class="feed-header">
  <div class="feed-tabs">
    <div class="feed-tablist" role="tablist" aria-label="Choose public feed scope">
      {#each scopes as scope (scope.value)}
        <button
          type="button"
          role="tab"
          aria-selected={feed.shownScope === scope.value}
          class:on={feed.shownScope === scope.value}
          onclick={() => void feed.setScope(scope.value)}>{scope.label}</button
        >
      {/each}
    </div>
    <button
      type="button"
      class="feed-tune"
      class:narrowed
      aria-label="Feed filters"
      aria-haspopup="dialog"
      aria-expanded={open}
      title="Feed filters"
      onclick={() => (open = true)}
    >
      <ToolbarIcon name="filter" />
    </button>
  </div>
  {#if feed.shownScope === 'region'}
    <!-- Region's place line (Polish Pass mockups): it and the filter button open one sheet. -->
    <button type="button" class="region-near" aria-haspopup="dialog" onclick={() => (open = true)}>
      <ToolbarIcon name="map-pin" />
      {#if feed.region.placeQuery}
        <span>Near <b>{feed.region.placeQuery}</b> · {radiusLabel}</span>
      {:else}
        <span>Choose a place to see what is near it</span>
      {/if}
    </button>
    {#if feed.region.message}
      <p class="inline-alert" role="alert">{feed.region.message}</p>
    {/if}
  {/if}
</header>

<FeedFilterSheet
  {open}
  title={`Show in ${scopeLabel}`}
  type={filter}
  sort={feed.sort}
  window={feed.window}
  onType={(v) => (filter = v)}
  onSort={(v) => void feed.setSort(v)}
  onWindow={(v) => void feed.setWindow(v)}
  onClose={() => (open = false)}
>
  {#snippet before()}
    {#if feed.shownScope === 'region'}
      <RegionControls region={feed.region} />
    {/if}
    {#if signedIn}
      <section>
        <h3>From</h3>
        <SegmentedChoice
          value={feed.audience}
          ariaLabel="Choose personal audience"
          options={audiences}
          onChange={(v) => void feed.setAudience(v)}
        />
      </section>
    {/if}
  {/snippet}
</FeedFilterSheet>
