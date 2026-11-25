<script lang="ts">
  import { t } from '#lib/i18n/index.ts';
  import EmptyState from '#lib/components/empty/EmptyState.svelte';
  import { untrack } from 'svelte';

  import AvatarBadge from '#lib/components/icons/AvatarBadge.svelte';
  import {
    formatSearchMeta,
    readSearchType,
    searchKindLabels,
    suggestedQueries,
    type SearchResultItem,
    type SearchType,
  } from '#lib/domain/search.ts';
  import { search } from '#lib/services/search.ts';
  import { openSearch } from '#lib/stores/searchOverlay.svelte.ts';

  import { goto } from '$app/navigation';
  import { resolve } from '$app/paths';

  import '#lib/components/styles/search.css';

  /** web's SearchPage: the query and kind come from the URL (`?q=`, `?type=`). */
  let { query, typeParam }: { query: string; typeParam: string | null } = $props();
  const type = $derived(readSearchType(typeParam));

  // web's options, in its order, for the kinds the app has (no activities yet; PARITY.md),
  // as a row of chips (Remaining Screens, 5).
  const typeOptions: { value: SearchType; label: string }[] = [
    { value: 'all', label: t('All') },
    { value: 'project', label: t('Projects') },
    { value: 'thread', label: t('Threads') },
    { value: 'event', label: t('Events') },
    { value: 'help_request', label: t('Requests') },
    { value: 'post', label: t('Posts') },
    { value: 'channel', label: t('Channels') },
    { value: 'community', label: t('Communities') },
    { value: 'user', label: t('People') },
  ];

  let draft = $state(untrack(() => query));
  let results = $state<SearchResultItem[]>([]);
  let loaded = $state(false);

  // A new ?q= or ?type= (submitting, a chip, the filter) loads its results.
  $effect(() => {
    const [q, t] = [query, type];
    untrack(() => {
      draft = q;
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

  function submit(event: SubmitEvent): void {
    event.preventDefault();
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- built from resolve('/search')
    void goto(searchHref(draft.trim(), type));
  }

  function open(href: string): void {
    // eslint-disable-next-line svelte/no-navigation-without-resolve -- the app's own paths
    void goto(href);
  }
</script>

<div class="search-page">
  <section class="page">
    <header class="page-header">
      <h1>{t('Search')}</h1>
      <form class="search-form" role="search" onsubmit={submit}>
        <div class="search-input-wrap">
          <svg aria-hidden="true" class="search-glyph" viewBox="0 0 24 24">
            <circle cx="11" cy="11" r="6" /><path d="m20 20-4.5-4.5" />
          </svg>
          <input
            aria-label={t('Search')}
            class="search-input"
            bind:value={draft}
            readonly
            onfocus={(event) => {
              event.currentTarget.blur();
              openSearch(draft);
            }}
            placeholder={t('Channels, communities, projects, usernames...')}
            type="search"
          />
        </div>
      </form>

      <div class="type-chips" role="radiogroup" aria-label={t('Filter search')}>
        {#each typeOptions as option (option.value)}
          <button
            role="radio"
            aria-checked={type === option.value}
            class="type-chip"
            class:on={type === option.value}
            type="button"
            onclick={() => open(searchHref(draft.trim() || query.trim(), option.value))}
            >{t(option.label)}</button
          >
        {/each}
      </div>

      {#if !query.trim()}
        <p class="search-lede">
          {t('Search channels, communities, projects, events, and profiles.')}
        </p>
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
        <EmptyState
          icon="search"
          title={t('No results for “{query}”', { query: query.trim() })}
          text={t('Try a channel name, community name, project title or username.')}
        />
      {:else if query.trim()}
        {#each results as result (result.id)}
          <!-- eslint-disable-next-line svelte/no-navigation-without-resolve -- the app's own paths -->
          <a class="result-card" class:person={result.kind === 'profile'} href={result.href}>
            {#if result.kind === 'profile'}<AvatarBadge
                size="md"
                name={result.title}
                agent={result.id}
              />{/if}
            <div class="result-copy">
              <div class="topline">
                <span class={`kind-chip kind-${result.kind}`}>{searchKindLabels[result.kind]}</span>
                {#if formatSearchMeta(result.meta, result.kind) !== searchKindLabels[result.kind]}
                  <span>{formatSearchMeta(result.meta, result.kind)}</span>
                {/if}
              </div>
              <h2>{result.title}</h2>
              {#if result.summary}<p>{result.summary}</p>{/if}
            </div>
          </a>
        {/each}
      {/if}
    </div>
  </section>
</div>
