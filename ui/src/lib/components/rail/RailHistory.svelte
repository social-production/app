<script lang="ts">
  import { railKicker, type RailItem } from '#lib/domain/activityRail.ts';
  import {
    historyMatches,
    historyOutcome,
    type HistoryFilter,
  } from '#lib/domain/activityRailHistory.ts';
  import { modeAccent } from '#lib/domain/projects.ts';

  /** web's History section: what ended, what you signed up for and the votes you cast. */
  let { items, onOpen }: { items: RailItem[]; onOpen: (item: RailItem) => void } = $props();

  let filter = $state<HistoryFilter>('all');
  const shown = $derived(items.filter((i) => historyMatches(i, filter)));
</script>

<details class="rail-section rail-section-history history-section">
  <summary class="rail-section-summary">
    <span class="summary-label">
      <span class="chevron" aria-hidden="true"></span>
      <span>History</span>
    </span>
  </summary>
  <div class="history-body">
    <label class="history-filter">
      <span class="history-filter-label">Show</span>
      <select aria-label="History filter" bind:value={filter}>
        <option value="all">All</option>
        <option value="votes">Votes</option>
        <option value="activities">Activities</option>
        <option value="help">Help</option>
        <option value="requests">Requests</option>
      </select>
    </label>
    {#if shown.length === 0}
      <p class="history-empty">No history items match this filter.</p>
    {:else}
      <div class="history-list">
        {#each shown as item (item.id)}
          <button
            class="history-row"
            style={`--row-accent: ${modeAccent(item.projectMode)};`}
            type="button"
            onclick={() => onOpen(item)}
          >
            <span class="history-kind">{railKicker(item)}</span>
            <strong>{item.title}</strong>
            {#if item.meta}
              <span class="history-meta">{item.meta}</span>
            {/if}
            <span
              class="history-outcome"
              class:passed={item.kind === 'vote' && item.outcome === 'passed'}
              class:failed={item.kind === 'vote' && item.outcome === 'failed'}
              class:participated={item.kind === 'project' && item.viewerParticipated}
            >
              {historyOutcome(item)}
            </span>
          </button>
        {/each}
      </div>
    {/if}
  </div>
</details>
