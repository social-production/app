<script lang="ts" module>
  export type FeedFilter = 'all' | 'projects';
</script>

<script lang="ts">
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import SegmentedChoice from '#lib/components/SegmentedChoice.svelte';
  import type { FeedScope, HomeAudience } from '#lib/domain/homeFeed.ts';
  import type { FeedSort, FeedWindow } from '#lib/domain/projects.ts';

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
    filter = $bindable(),
  }: { feed: HomeFeedState; signedIn: boolean; filter: FeedFilter } = $props();

  let open = $state(false);

  const scopes: { value: FeedScope; label: string }[] = [
    { value: 'home', label: 'Home' },
    { value: 'global', label: 'Global' },
    { value: 'region', label: 'Region' },
  ];
  const audiences: { value: HomeAudience; label: string }[] = [
    { value: 'following', label: 'Following only' },
    { value: 'popular', label: 'Following + popular' },
  ];
  // web's filter menu; threads, events and help requests join it as they land.
  const filters: { value: FeedFilter; label: string }[] = [
    { value: 'all', label: 'All items' },
    { value: 'projects', label: 'Projects' },
  ];
  const sorts: { value: FeedSort; label: string }[] = [
    { value: 'trending', label: 'Trending' },
    { value: 'recent', label: 'Most recent' },
  ];
  const windows: { value: FeedWindow; label: string }[] = [
    { value: 'today', label: 'Today' },
    { value: 'week', label: 'This week' },
    { value: 'month', label: 'This month' },
    { value: 'all', label: 'All time' },
  ];

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
    <div class="controls-row feed-region-row"><RegionControls region={feed.region} /></div>
    {#if feed.region.message}
      <p class="inline-alert" role="alert">{feed.region.message}</p>
    {/if}
    {#if feed.region.suggestions.length > 0}
      <ul class="suggestions">
        {#each feed.region.suggestions as place, i (i)}
          <li>
            <button
              type="button"
              onmousedown={(e) => e.preventDefault()}
              onclick={() => void feed.region.select(place)}>{place.displayLabel}</button
            >
          </li>
        {/each}
      </ul>
    {/if}
  {/if}
</header>

<OverlaySheet
  {open}
  title={`Show in ${scopeLabel}`}
  labelledById="feed-filter-title"
  rootClass="shell feed-filter-sheet"
  onClose={() => (open = false)}
>
  <div class="feed-filter-groups">
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
    <section>
      <h3>Type</h3>
      <SegmentedChoice
        value={filter}
        ariaLabel="Filter public feed"
        options={filters}
        onChange={(v) => (filter = v)}
      />
    </section>
    <section>
      <h3>Order</h3>
      <SegmentedChoice
        value={feed.sort}
        ariaLabel="Sort public feed by"
        options={sorts}
        onChange={(v) => void feed.setSort(v)}
      />
    </section>
    <section>
      <h3>When</h3>
      <SegmentedChoice
        value={feed.window}
        ariaLabel="Public feed time window"
        options={windows}
        onChange={(v) => void feed.setWindow(v)}
      />
    </section>
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-submit" type="button" onclick={() => (open = false)}>Show results</button
      >
    </div>
  {/snippet}
</OverlaySheet>
