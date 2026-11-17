<script lang="ts">
  import type { Snippet } from 'svelte';

  import OverlaySheet from '#lib/components/overlay/OverlaySheet.svelte';
  import PillChoice from '#lib/components/PillChoice.svelte';
  import SegmentedChoice from '#lib/components/SegmentedChoice.svelte';
  import { FEED_SORTS, FEED_TYPES, FEED_WINDOWS, type FeedType } from '#lib/domain/feedTypes.ts';
  import type { FeedSort, FeedWindow } from '#lib/domain/projects.ts';

  /**
   * The feed's filter sheet (Polish Pass §2, Scope Filters §1): Type, Order and When, one line
   * each, for Home, channels, communities, the Platform and profiles. `before` holds a feed's
   * own groups (Home's From, Region's place).
   */
  interface Props {
    open: boolean;
    title: string;
    type: FeedType;
    sort: FeedSort;
    window: FeedWindow;
    sorts?: { value: FeedSort; label: string }[];
    onType: (type: FeedType) => void;
    onSort: (sort: FeedSort) => void;
    onWindow: (window: FeedWindow) => void;
    onClose: () => void;
    before?: Snippet;
    after?: Snippet;
  }

  let {
    open,
    title,
    type,
    sort,
    window,
    sorts = FEED_SORTS,
    onType,
    onSort,
    onWindow,
    onClose,
    before,
    after,
  }: Props = $props();
</script>

<OverlaySheet
  {open}
  {title}
  labelledById="feed-filter-title"
  rootClass="shell feed-filter-sheet"
  {onClose}
>
  <div class="feed-filter-groups">
    {#if before}{@render before()}{/if}
    <section>
      <h3>Type</h3>
      <PillChoice
        value={type}
        ariaLabel="Filter public feed"
        options={FEED_TYPES}
        onChange={onType}
      />
    </section>
    <section>
      <h3>Order</h3>
      <SegmentedChoice
        value={sort}
        ariaLabel="Sort public feed by"
        options={sorts}
        onChange={onSort}
      />
    </section>
    <section>
      <h3>When</h3>
      <SegmentedChoice
        value={window}
        ariaLabel="Public feed time window"
        options={FEED_WINDOWS}
        onChange={onWindow}
      />
    </section>
    {#if after}{@render after()}{/if}
  </div>
  {#snippet footer()}
    <div class="sheet-actions">
      <button class="sheet-submit" type="button" onclick={onClose}>Show results</button>
    </div>
  {/snippet}
</OverlaySheet>
