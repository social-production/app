<script lang="ts">
  import { untrack } from 'svelte';

  import IconMenuButton from '#lib/components/IconMenuButton.svelte';
  import type { SurfaceIconId } from '#lib/components/icons/surface.ts';
  import ToolbarIcon from '#lib/components/icons/ToolbarIcon.svelte';
  import SearchSuggestionsList from '#lib/components/search/SearchSuggestionsList.svelte';
  import {
    formatSearchMeta,
    readSearchType,
    searchKindLabels,
    suggestedQueries,
    type SearchResultItem,
    type SearchType,
  } from '#lib/domain/search.ts';
  import { search } from '#lib/services/search.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import { createLiveSearchScheduler } from './liveSearch.ts';

  import '#lib/components/styles/search.css';

  /** web's SearchPage: the query and kind come from the URL (`?q=`, `?type=`). */
  let { query, typeParam }: { query: string; typeParam: string | null } = $props();
  const type = $derived(readSearchType(typeParam));

  // web's options, in its order, for the kinds the app has (no threads, activities or
  // posts yet; PARITY.md).
  const typeOptions: { value: SearchType; label: string; icon?: SurfaceIconId }[] = [
    { value: 'all', label: 'All' },
    { value: 'project', label: 'Projects', icon: 'project' },
    { value: 'event', label: 'Events', icon: 'event' },
    { value: 'help_request', label: 'Help requests', icon: 'help-request' },
    { value: 'channel', label: 'Channels', icon: 'channel' },
    { value: 'community', label: 'Communities', icon: 'community' },
    { value: 'user', label: 'Users', icon: 'user' },
  ];

  const liveSearch = createLiveSearchScheduler();
  let draft = $state(untrack(() => query));
  let results = $state<SearchResultItem[]>([]);
  let loaded = $state(false);
  let liveResults = $state<SearchResultItem[]>([]);
  let liveLoading = $state(false);
  let showSuggestions = $state(false);

  // A new ?q= or ?type= (submitting, a chip, the filter) loads its results.
  $effect(() => {
    const [q, t] = [query, type];
    untrack(() => {
      draft = q;
      showSuggestions = false;
      loaded = false;
      void search(q, t).then(
        (found) => ((results = found), (loaded = true)),
        () => ((results = []), (loaded = true)),
      );
    });
  });

  function searchHref(q: string, t: SearchType): string {
    const parts = [
      ...(q ? [`q=${encodeURIComponent(q)}`] : []),
      ...(t !== 'all' ? [`type=${t}`] : []),
    ];
    return parts.length ? `${resolve('/search')}?${parts.join('&')}` : resolve('/search');
  }

  function live(): void {
    showSuggestions = true;
    liveSearch.schedule(draft, type, (found, loading) => {
      liveResults = found;
      liveLoading = loading;
    });
  }

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    showSuggestions = false;
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- built from resolve('/search')
    void goto(searchHref(draft.trim(), type));
  }

  function open(href: string): void {
    showSuggestions = false;
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- the app's own paths
    void goto(href);
  }
</script>

<div class="search-page">
  <section class="page">
    <header class="page-header">
      <h1>Search</h1>
      {#if query.trim()}
        <p>Showing results for "{query}".</p>
      {:else}
        <p>Search channels, communities, projects, events, and profiles.</p>
      {/if}
      <form class="search-form" role="search" onsubmit={submit}>
        <div class="search-input-wrap">
          <input
            aria-label="Search"
            class="search-input"
            bind:value={draft}
            oninput={live}
            onfocus={() => draft.trim() && live()}
            onblur={() => setTimeout(() => (showSuggestions = false), 150)}
            placeholder="Channels, communities, projects, usernames..."
            type="search"
          />
          {#if showSuggestions && draft.trim()}
            <SearchSuggestionsList
              loading={liveLoading}
              overlay
              results={liveResults}
              onSelect={open}
            />
          {/if}
        </div>
        <button class="primary-button" type="submit">Search</button>
      </form>

      <div class="filter-row">
        <IconMenuButton
          value={type}
          ariaLabel="Filter search"
          defaultValue="all"
          options={typeOptions}
          showOptionIcons
          showTriggerLabel
          onChange={(next) => open(searchHref(draft.trim() || query.trim(), next))}
        >
          <ToolbarIcon name="filter" />
        </IconMenuButton>
      </div>

      {#if !query.trim()}
        <div class="chip-row">
          {#each suggestedQueries as chip (chip)}
            <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- built from resolve('/search') -->
            <a class="query-chip" href={searchHref(chip, type)}>{chip}</a>
          {/each}
        </div>
      {/if}
    </header>

    <div class="stack">
      {#if query.trim() && loaded && results.length === 0}
        <section class="result-card">
          <h2>No results</h2>
          <p>Try a channel name, community name, project title, or username.</p>
        </section>
      {:else if query.trim()}
        {#each results as result (result.id)}
          <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- the app's own paths -->
          <a class="result-card" href={result.href}>
            <div class="topline">
              <span class="kind-chip">{searchKindLabels[result.kind]}</span>
              <span>{formatSearchMeta(result.meta, result.kind)}</span>
            </div>
            <h2>{result.title}</h2>
            <p>{result.summary}</p>
          </a>
        {/each}
      {/if}
    </div>
  </section>
</div>
