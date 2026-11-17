<script lang="ts">
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import { feedSummary, type FeedType } from '#lib/domain/feedTypes.ts';
  import type { FeedWindow } from '#lib/domain/projects.ts';

  import type { FeedState } from './feedState.svelte.ts';
  import FeedFilterSheet from './FeedFilterSheet.svelte';

  /**
   * A channel's, community's or the Platform's feed bar (Scope Filters mockups §1a): the
   * current choice, and the filter button opening the same sheet as Home. The dot shows when
   * the feed is narrower than everything.
   */
  let {
    feed,
    name,
    type = $bindable(),
  }: { feed: FeedState; name: string; type: FeedType } = $props();

  let open = $state(false);
  const narrowed = $derived(type !== 'all' || feed.window !== 'all');

  function reload(change: () => void): void {
    change();
    void feed.reload();
  }
</script>

<div class="feed-filter-bar">
  <span>{feedSummary(feed.sort, feed.window, type)}</span>
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

<FeedFilterSheet
  {open}
  title={`Show in ${name}`}
  {type}
  sort={feed.sort}
  window={feed.window}
  onType={(v) => (type = v)}
  onSort={(v) => reload(() => (feed.sort = v))}
  onWindow={(v: FeedWindow) => reload(() => (feed.window = v))}
  onClose={() => (open = false)}
/>
