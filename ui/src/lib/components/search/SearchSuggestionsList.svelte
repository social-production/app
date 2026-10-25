<script lang="ts">
  import { formatSearchMeta, searchKindLabels, type SearchResultItem } from '#lib/domain/search.ts';

  /** web's SearchSuggestionsList: live matches under the search box. */
  interface Props {
    results: SearchResultItem[];
    loading?: boolean;
    emptyMessage?: string;
    onSelect: (href: string) => void;
    overlay?: boolean;
  }

  let {
    results,
    loading = false,
    emptyMessage = 'No matches yet. Try a channel, project, or username.',
    onSelect,
    overlay = false,
  }: Props = $props();
</script>

{#if loading}
  <div class="suggestions-panel" class:overlay role="status">
    <p>Searching…</p>
  </div>
{:else if results.length > 0}
  <div class="suggestions-panel" class:overlay role="listbox">
    {#each results as result (result.id)}
      <button class="suggestion-row" type="button" onclick={() => onSelect(result.href)}>
        <span class="kind-chip">{searchKindLabels[result.kind]}</span>
        <span class="suggestion-copy">
          <strong>{result.title}</strong>
          <span>{formatSearchMeta(result.meta, result.kind)}</span>
        </span>
      </button>
    {/each}
  </div>
{:else}
  <div class="suggestions-panel muted" class:overlay role="status">
    <p>{emptyMessage}</p>
  </div>
{/if}
